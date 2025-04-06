use std::path::PathBuf;

use crate::Markdown;

pub trait BlogRepository {
    fn get_all_posts(&self) -> Result<Vec<Markdown>, RepositoryError>;
    fn find_post_by_slug(&self, slug: &str) -> Result<Option<Markdown>, RepositoryError>;
    fn get_page(&self, slug: &str) -> Result<Option<Markdown>, RepositoryError>;
}

pub enum RepositoryError {
    NotFound,
    UnexpectedError,
}

pub(crate) struct FileSystemBlogRepository {
    content_dir: PathBuf,
}

impl FileSystemBlogRepository {
    pub fn new(content_dir: PathBuf) -> Self {
        FileSystemBlogRepository { content_dir }
    }

    fn posts_dir(&self) -> PathBuf {
        self.content_dir.join("posts")
    }

    fn pages_dir(&self) -> PathBuf {
        self.content_dir.join("pages")
    }

    fn all_posts_unsorted(&self) -> Result<Vec<Markdown>, RepositoryError> {
        let posts_dir = self.posts_dir();
        if !posts_dir.exists() {
            return Ok(Vec::new());
        }
        let mut markdowns = Vec::new();
        for entry in std::fs::read_dir(posts_dir).map_err(|_| RepositoryError::NotFound)? {
            let entry = entry.map_err(|_| RepositoryError::UnexpectedError)?;
            let path = entry.path();

            if path.extension().map_or(true, |ext| ext != "md") {
                continue;
            }

            let content = read_to_string(path.clone())?;
            let mut markdown = Markdown::from_str(&content);
            if markdown.repo_slug.is_none() {
                markdown.repo_slug = path.file_stem().map(|s| s.to_string_lossy().into_owned())
            }

            markdowns.push(markdown);
        }
        Ok(markdowns)
    }
}

impl BlogRepository for FileSystemBlogRepository {
    fn get_all_posts(&self) -> Result<Vec<Markdown>, RepositoryError> {
        let mut markdowns = self.all_posts_unsorted()?;

        fn sort_by_newest_first(
            date_a: &chrono::NaiveDate,
            date_b: &chrono::NaiveDate,
        ) -> std::cmp::Ordering {
            date_b.cmp(date_a)
        }

        fn sort_by_title_alphabetically(a: &Markdown, b: &Markdown) -> std::cmp::Ordering {
            a.title.cmp(&b.title)
        }
        markdowns.sort_by(|a, b| match (&a.publish_date, &b.publish_date) {
            (Some(date_a), Some(date_b)) => sort_by_newest_first(date_a, date_b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => sort_by_title_alphabetically(a, b),
        });

        Ok(markdowns)
    }

    fn find_post_by_slug(&self, slug: &str) -> Result<Option<Markdown>, RepositoryError> {
        let markdowns = self.all_posts_unsorted()?;
        Ok(markdowns
            .into_iter()
            .find(|markdown| markdown.slug() == slug))
    }

    fn get_page(&self, slug: &str) -> Result<Option<Markdown>, RepositoryError> {
        let page_path = self.pages_dir().join(format!("{}.md", &slug));

        if page_path.exists() {
            let content = read_to_string(page_path.clone())?;
            Ok(Some(Markdown {
                title: None,
                content,
                repo_slug: Some(slug.to_string()),
                publish_date: None,
            }))
        } else {
            Ok(None)
        }
    }
}

fn read_to_string(page_path: PathBuf) -> Result<String, RepositoryError> {
    std::fs::read_to_string(page_path).map_err(|_| RepositoryError::UnexpectedError)
}
