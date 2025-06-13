use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub struct FrontMatter {
    pub title: Option<String>,
    #[serde(alias = "datePublished")]
    pub publish_date: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
}

#[derive(Clone)]
pub struct Markdown {
    pub title: Option<String>,
    pub content: String,
    pub slugs: Vec<String>,
    pub publish_date: Option<chrono::NaiveDate>,
}

pub struct ParsedContent {
    pub front_matter: Option<FrontMatter>,
    pub content: String,
}

#[derive(serde::Serialize, Debug)]
pub struct BlogPost {
    pub title: String,
    pub publish_date: Option<String>,
    pub slug: String,
}

impl Markdown {
    pub fn from_str(text: &str) -> Self {
        let parsed = Self::parse_front_matter(text);
        match parsed.front_matter {
            Some(front_matter) => Markdown {
                title: front_matter.title,
                content: parsed.content,
                publish_date: front_matter
                    .publish_date
                    .and_then(|s| parse_date_for_sorting(s.as_str())),
                slugs: front_matter.slug.into_iter().collect(),
            },
            None => Markdown {
                title: None,
                content: parsed.content,
                publish_date: None,
                slugs: vec![],
            },
        }
    }

    pub fn contains(&self, slug: String) -> bool {
        self.slugs.contains(&slug)
    }

    // First frontmatter then filename.
    pub fn primary_slug(&self) -> String {
        self.slugs
            .clone()
            .first()
            .expect("Markdown from repo should always contain slug") // todo: make self.title String?
            .to_string()
    }

    fn parse_front_matter(content: &str) -> ParsedContent {
        use gray_matter::engine::YAML;
        use gray_matter::Matter;

        let matter = Matter::<YAML>::new();
        let result = matter.parse(content);

        let yaml_text = result.matter;
        let content = result.content;

        let front_matter = match serde_yaml::from_str::<FrontMatter>(yaml_text.as_str()) {
            Ok(front_matter) => Some(front_matter),
            Err(e) => {
                eprintln!("Error parsing front matter: {}", e);
                None
            }
        };

        ParsedContent {
            front_matter,
            content,
        }
    }
}

pub fn parse_date_for_sorting(date_str: &str) -> Option<chrono::NaiveDate> {
    // First try the JavaScript date format (e.g., "Fri Dec 06 2024 12:36:53 GMT+0000")
    if let Ok(datetime) = chrono::DateTime::parse_from_str(
        // Remove the (Coordinated Universal Time) part if present
        date_str.split(" (").next().unwrap_or(date_str),
        "%a %b %d %Y %H:%M:%S GMT%z",
    ) {
        let result = datetime.date_naive();
        return Some(result);
    }

    // Try simple YYYY-MM-DD format
    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Some(date);
    }

    // Try "Month Day, Year" format (e.g., "Dec 6, 2024")
    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%b %d, %Y") {
        return Some(date);
    }

    // Try with full month name "Month Day, Year" format (e.g., "December 6, 2024")
    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%B %d, %Y") {
        return Some(date);
    }

    None
}

pub fn format_date_for_post_view(date: chrono::NaiveDate) -> String {
    date.format("%B %d, %Y").to_string()
}

pub fn format_date_for_posts_overview(date: chrono::NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_date_str(date_str: &str) -> String {
        if let Some(date) = parse_date_for_sorting(date_str) {
            return format_date_for_post_view(date);
        }
        date_str.to_string()
    }

    #[test]
    fn test_format_date_js_format() {
        // JavaScript date format
        let input = "Fri Dec 06 2024 12:36:53 GMT+0000 (Coordinated Universal Time)";
        let expected = "December 06, 2024";
        assert_eq!(format_date_str(input), expected);
    }

    #[test]
    fn test_format_date_js_format_without_timezone_name() {
        // JavaScript date format without timezone name in parentheses
        let input = "Fri Dec 06 2024 12:36:53 GMT+0000";
        let expected = "December 06, 2024";
        assert_eq!(format_date_str(input), expected);
    }

    #[test]
    fn test_format_date_iso_format() {
        // Simple ISO format
        let input = "2024-12-06";
        let expected = "December 06, 2024";
        assert_eq!(format_date_str(input), expected);
    }

    #[test]
    fn test_format_date_invalid_format() {
        // Invalid format should return the original string
        let input = "Invalid date";
        let expected = "Invalid date";
        assert_eq!(format_date_str(input), expected);
    }

    #[test]
    fn test_format_date_mixed_format() {
        // Another common format
        let input = "Dec 6, 2024";

        // With our new implementation, this format is now supported
        let expected = "December 06, 2024";
        assert_eq!(format_date_str(input), expected);
    }

    #[test]
    fn test_format_date_consistency() {
        let test_dates = [
            "2023-01-15",
            "Fri Dec 06 2024 12:36:53 GMT+0000",
            "Mar 10, 2023",
            "January 20, 2024",
        ];

        for date in test_dates {
            let formatted = format_date_str(date);
            assert!(parse_date_for_sorting(&formatted).is_some());
        }
    }
}
