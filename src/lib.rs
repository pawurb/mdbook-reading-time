use mdbook_preprocessor::{
    book::{Book, BookItem, Chapter},
    errors::Result,
    Preprocessor, PreprocessorContext,
};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct ReadingTime;

impl ReadingTime {
    pub fn new() -> ReadingTime {
        ReadingTime
    }
}

static WORDS_PER_MINUTE: usize = 200;

impl Preprocessor for ReadingTime {
    fn name(&self) -> &str {
        "reading-time"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book> {
        let words_per_minute: usize = ctx
            .config
            .get::<usize>("preprocessor.reading-time.words-per-minute")
            .ok()
            .flatten()
            .unwrap_or(WORDS_PER_MINUTE);

        let mut book = book;
        book.for_each_mut(|item: &mut BookItem| {
            if let BookItem::Chapter(ref mut chapter) = *item {
                let _ = handle_chapter(chapter, words_per_minute);
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool> {
        Ok(renderer != "not-supported")
    }
}

fn handle_chapter(chapter: &mut Chapter, words_per_minute: usize) -> Result<()> {
    let content = chapter.content.as_str();
    let word_count = content.unicode_words().count();
    let reading_time = word_count / words_per_minute;
    let minutes = if reading_time == 1 {
        "minute"
    } else {
        "minutes"
    };

    chapter.content = chapter
        .content
        .replace("{{ #word_count }}", word_count.to_string().as_str())
        .replace(
            "{{ #reading_time }}",
            &format!("{} {minutes}", reading_time.to_string().as_str()),
        );
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reading_preprocessor_run() {
        let input_json = r##"[
                {
                    "root": "/path/to/book",
                    "config": {
                        "book": {
                            "authors": ["AUTHOR"],
                            "language": "en",
                            "src": "src",
                            "title": "TITLE"
                        },
                        "preprocessor": {
                            "reading-time": {}
                        }
                    },
                    "renderer": "html",
                    "mdbook_version": "0.5.1"
                },
                {
                    "items": [
                        {
                            "Chapter": {
                                "name": "Chapter 1",
                                "content": "# Chapter 1\n {{ #word_count }}\n\n{{ #reading_time }}",
                                "number": [1],
                                "sub_items": [],
                                "path": "chapter_1.md",
                                "source_path": "chapter_1.md",
                                "parent_names": []
                            }
                        }
                    ],
                    "__non_exhaustive": null
                }
            ]"##;
        let input_json = input_json.as_bytes();

        let (ctx, book) = mdbook_preprocessor::parse_input(input_json).unwrap();
        let result = ReadingTime::new().run(&ctx, book);
        assert!(result.is_ok());

        let actual_book = result.unwrap();
        let chapter = actual_book.iter().next().unwrap();

        match chapter {
            BookItem::Chapter(chapter) => {
                assert_eq!(chapter.content, "# Chapter 1\n 4\n\n0 minutes");
            }
            _ => panic!("Expected a chapter"),
        };
    }
}
