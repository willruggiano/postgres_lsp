use pg_console::MarkupBuf;
use pg_diagnostics::{
    advice::CodeSuggestionAdvice, category, Advices, Category, Diagnostic, DiagnosticExt,
    DiagnosticTags, Error, Location, Severity, Visit,
};
use text_size::TextRange;
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

use crate::rule::RuleDiagnostic;

/// Small wrapper for diagnostics during the analysis phase.
///
/// During these phases, analyzers can create various type diagnostics and some of them
/// don't have all the info to actually create a real [Diagnostic].
///
/// This wrapper serves as glue, which eventually is able to spit out full fledged diagnostics.
///
#[derive(Debug)]
pub struct AnalyzerDiagnostic {
    kind: DiagnosticKind,
}

impl From<RuleDiagnostic> for AnalyzerDiagnostic {
    fn from(rule_diagnostic: RuleDiagnostic) -> Self {
        Self {
            kind: DiagnosticKind::Rule(rule_diagnostic),
        }
    }
}

#[derive(Debug)]
enum DiagnosticKind {
    /// It holds various info related to diagnostics emitted by the rules
    Rule(RuleDiagnostic),
    /// We have raw information to create a basic [Diagnostic]
    Raw(Error),
}

impl Diagnostic for AnalyzerDiagnostic {
    fn category(&self) -> Option<&'static Category> {
        match &self.kind {
            DiagnosticKind::Rule(rule_diagnostic) => Some(rule_diagnostic.category),
            DiagnosticKind::Raw(error) => error.category(),
        }
    }
    fn description(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            DiagnosticKind::Rule(rule_diagnostic) => Debug::fmt(&rule_diagnostic.message, fmt),
            DiagnosticKind::Raw(error) => error.description(fmt),
        }
    }

    fn message(&self, fmt: &mut pg_console::fmt::Formatter<'_>) -> std::io::Result<()> {
        match &self.kind {
            DiagnosticKind::Rule(rule_diagnostic) => {
                pg_console::fmt::Display::fmt(&rule_diagnostic.message, fmt)
            }
            DiagnosticKind::Raw(error) => error.message(fmt),
        }
    }

    fn severity(&self) -> Severity {
        match &self.kind {
            DiagnosticKind::Rule { .. } => Severity::Error,
            DiagnosticKind::Raw(error) => error.severity(),
        }
    }

    fn tags(&self) -> DiagnosticTags {
        match &self.kind {
            DiagnosticKind::Rule(rule_diagnostic) => rule_diagnostic.tags,
            DiagnosticKind::Raw(error) => error.tags(),
        }
    }

    fn location(&self) -> Location<'_> {
        match &self.kind {
            DiagnosticKind::Rule(rule_diagnostic) => {
                Location::builder().span(&rule_diagnostic.span).build()
            }
            DiagnosticKind::Raw(error) => error.location(),
        }
    }

    fn advices(&self, visitor: &mut dyn Visit) -> std::io::Result<()> {
        match &self.kind {
            DiagnosticKind::Rule(rule_diagnostic) => rule_diagnostic.advices().record(visitor)?,
            DiagnosticKind::Raw(error) => error.advices(visitor)?,
        }

        Ok(())
    }
}

impl AnalyzerDiagnostic {
    /// Creates a diagnostic from a generic [Error]
    pub fn from_error(error: Error) -> Self {
        Self {
            kind: DiagnosticKind::Raw(error),
        }
    }

    pub fn get_span(&self) -> Option<TextRange> {
        match &self.kind {
            DiagnosticKind::Rule(rule_diagnostic) => rule_diagnostic.span,
            DiagnosticKind::Raw(error) => error.location().span,
        }
    }

    pub const fn is_raw(&self) -> bool {
        matches!(self.kind, DiagnosticKind::Raw(_))
    }
}

#[derive(Debug, Diagnostic, Clone)]
#[diagnostic(severity = Warning)]
pub struct SuppressionDiagnostic {
    #[category]
    category: &'static Category,
    #[location(span)]
    range: TextRange,
    #[message]
    #[description]
    message: String,
    #[tags]
    tags: DiagnosticTags,
}

impl SuppressionDiagnostic {
    pub(crate) fn new(
        category: &'static Category,
        range: TextRange,
        message: impl Display,
    ) -> Self {
        Self {
            category,
            range,
            message: message.to_string(),
            tags: DiagnosticTags::empty(),
        }
    }

    pub(crate) fn with_tags(mut self, tags: DiagnosticTags) -> Self {
        self.tags |= tags;
        self
    }
}

