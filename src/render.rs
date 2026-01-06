use anyhow::Context;
use chrono::{Datelike, Duration, Local, Timelike, Utc};
use image::{ImageBuffer, Rgba};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_render::render;

static DEJAVUSANS_FONT: Lazy<Bytes> =
    Lazy::new(|| Bytes::new(include_bytes!("../assets/DejaVuSans.ttf").as_slice()));
static NOTOEMOJI_FONT: Lazy<Bytes> =
    Lazy::new(|| Bytes::new(include_bytes!("../assets/NotoColorEmoji.ttf").as_slice()));

pub fn render_widget(
    document: &str,
    pixel_per_pt: f32,
) -> anyhow::Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let world = MemoryWorld::new(document)?;
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
}

impl MemoryWorld {
    fn new(source_text: &str) -> anyhow::Result<Self> {
        let main_id = FileId::new(None, VirtualPath::new("main.typ"));
        let source = Source::new(main_id, source_text.to_string());

        let mut fonts: Vec<Font> = Font::iter(DEJAVUSANS_FONT.clone()).collect();
        fonts.extend(Font::iter(NOTOEMOJI_FONT.clone()));
        let book = LazyHash::new(FontBook::from_fonts(fonts.iter()));

        let library = LazyHash::new(Library::builder().build());

        Ok(Self {
            source,
            library,
            book,
            fonts,
        })
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
            Err(FileError::NotFound(PathBuf::from(
                id.vpath().as_rooted_path(),
            )))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.source.id() {
            Ok(Bytes::from_string(self.source.text().to_string()))
        } else {
            Err(FileError::NotFound(PathBuf::from(
                id.vpath().as_rooted_path(),
            )))
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
