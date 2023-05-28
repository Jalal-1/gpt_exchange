//! Page contexts for rendering.

use derive_more::Constructor;
use serde::Serialize;

/// Must be implemented by all structures which are used as a [`Context`](handlebars::Context) for template rendering.
pub trait PageContext {
    /// The title of the page.
    fn title(&self) -> &str;
    /// The path to the template.
    ///
    /// Paths start at the root of the `templates` directory, and **do not** contain the file extension.
    fn template_path(&self) -> &str;
    /// The "parent" or "base" `template` path to be utilized for the page.
    fn parent(&self) -> &str;
}

/// The Home page.
#[derive(Debug, Serialize)]
pub struct Home {}

impl Default for Home {
    fn default() -> Self {
        Self {}
    }
}

impl PageContext for Home {
    fn template_path(&self) -> &str {
        "home"
    }
    fn title(&self) -> &str {
        "See Jobs!"
    }
    fn parent(&self) -> &str {
        "base"
    }
}

/// The page for viewing a [`Job`](crate::Job).
#[derive(Debug, Serialize, Constructor)]
pub struct ViewJob {
    pub job: crate::Job,
}

impl PageContext for ViewJob {
    fn template_path(&self) -> &str {
        "job"
    }
    fn title(&self) -> &str {
        "View Job"
    }
    fn parent(&self) -> &str {
        "base"
    }
}

/// The page to enter a [`Password`](crate::domain::job::field::Password) for a protected [`Job`](crate::Job);
#[derive(Debug, Serialize, Constructor)]
pub struct PasswordRequired {
    shortcode: crate::ShortCode,
}

impl PageContext for PasswordRequired {
    fn template_path(&self) -> &str {
        "job_need_password"
    }
    fn title(&self) -> &str {
        "Password Required"
    }
    fn parent(&self) -> &str {
        "base"
    }
}
