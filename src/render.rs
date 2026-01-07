use anyhow::Context;
use chrono::{Datelike, Duration, Local, Timelike, Utc};
use image::{ImageBuffer, Rgba};
use once_cell::sync::Lazy;
use std::fs;
use std::path::{Path, PathBuf};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_render::render;

static DEJAVUSANS_FONT: Lazy<Bytes> =
    Lazy::new(|| Bytes::new(include_bytes!("../assets/DejaVuSans.ttf").as_slice()));
static DEJAVUSANS_BOLD_FONT: Lazy<Bytes> =
    Lazy::new(|| Bytes::new(include_bytes!("../assets/DejaVuSans-Bold.ttf").as_slice()));
static NOTOEMOJI_FONT: Lazy<Bytes> =
    Lazy::new(|| Bytes::new(include_bytes!("../assets/NotoEmoji-Regular.ttf").as_slice()));

pub fn render_widget(
    document: &str,
    main_path: &str,
    template_root: &Path,
    pixel_per_pt: f32,
) -> anyhow::Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let world = MemoryWorld::new(document, main_path, template_root)?;
    let warned = typst::compile::<typst::layout::PagedDocument>(&world);

    if !warned.warnings.is_empty() {
        for warning in warned.warnings {
            tracing::warn!(?warning, "typst warning while compiling widget");
        }
    }

    let document = warned
        .output
        .map_err(|errors| anyhow::anyhow!("typst errors: {errors:?}"))?;
    let pixmap = render(&document.pages[0], pixel_per_pt);

    ImageBuffer::from_vec(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
        .context("failed to build image buffer from typst pixmap")
}

struct MemoryWorld {
    source: Source,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    template_root: PathBuf,
}

impl MemoryWorld {
    fn new(source_text: &str, main_path: &str, template_root: &Path) -> anyhow::Result<Self> {
        let main_id = FileId::new(None, VirtualPath::new(main_path));
        let source = Source::new(main_id, source_text.to_string());

        let mut fonts: Vec<Font> = Font::iter(DEJAVUSANS_FONT.clone()).collect();
        fonts.extend(Font::iter(DEJAVUSANS_BOLD_FONT.clone()));
        fonts.extend(Font::iter(NOTOEMOJI_FONT.clone()));

        let book = LazyHash::new(FontBook::from_fonts(fonts.iter()));

        let library = LazyHash::new(Library::builder().build());

        Ok(Self {
            source,
            library,
            book,
            fonts,
            template_root: template_root.to_path_buf(),
        })
    }

    fn resolve_path(&self, vpath: &VirtualPath) -> PathBuf {
        let rooted = vpath.as_rooted_path();
        let relative = rooted.strip_prefix("/").unwrap_or(rooted);

        self.template_root.join(relative)
    }
}

impl World for MemoryWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            let path = self.resolve_path(id.vpath());
            fs::read_to_string(&path)
                .map(|contents| Source::new(id, contents))
                .map_err(|_| FileError::NotFound(path))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.source.id() {
            Ok(Bytes::from_string(self.source.text().to_string()))
        } else {
            let path = self.resolve_path(id.vpath());
            let data = fs::read(&path).map_err(|_| FileError::NotFound(path.clone()))?;

            Ok(Bytes::new(data))
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let now = match offset {
            Some(hours) => Utc::now() + Duration::hours(hours),
            None => Local::now().with_timezone(&Utc),
        };

        Datetime::from_ymd_hms(
            now.year(),
            now.month() as u8,
            now.day() as u8,
            now.hour() as u8,
            now.minute() as u8,
            now.second() as u8,
        )
    }
}
