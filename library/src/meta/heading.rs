use typst::font::FontWeight;
use typst::util::option_eq;

use super::{Counter, CounterUpdate, LocalName, Numbering, Outlinable, Refable};
use crate::layout::{BlockElem, HElem, VElem};
use crate::meta::{Count, Supplement};
use crate::prelude::*;
use crate::text::{SpaceElem, TextElem, TextSize};

/// A section heading.
///
/// With headings, you can structure your document into sections. Each heading
/// has a _level,_ which starts at one and is unbounded upwards. This level
/// indicates the logical role of the following content (section, subsection,
/// etc.)  A top-level heading indicates a top-level section of the document
/// (not the document's title).
///
/// Typst can automatically number your headings for you. To enable numbering,
/// specify how you want your headings to be numbered with a
/// [numbering pattern or function]($func/numbering).
///
/// Independently from the numbering, Typst can also automatically generate an
/// [outline]($func/outline) of all headings for you. To exclude one or more
/// headings from this outline, you can set the `outlined` parameter to
/// `{false}`.
///
/// ## Example { #example }
/// ```example
/// #set heading(numbering: "1.a)")
///
/// = Introduction
/// In recent years, ...
///
/// == Preliminaries
/// To start, ...
/// ```
///
/// ## Syntax { #syntax }
/// Headings have dedicated syntax: They can be created by starting a line with
/// one or multiple equals signs, followed by a space. The number of equals
/// signs determines the heading's logical nesting depth.
///
/// Display: Heading
/// Category: meta
#[element(Locatable, Synthesize, Count, Show, Finalize, LocalName, Refable, Outlinable)]
pub struct HeadingElem {
    /// The logical nesting depth of the heading, starting from one.
    #[default(NonZeroUsize::ONE)]
    pub level: NonZeroUsize,

    /// How to number the heading. Accepts a
    /// [numbering pattern or function]($func/numbering).
    ///
    /// ```example
    /// #set heading(numbering: "1.a.")
    ///
    /// = A section
    /// == A subsection
    /// === A sub-subsection
    /// ```
    pub numbering: Option<Numbering>,

    /// A supplement for the heading.
    ///
    /// For references to headings, this is added before the referenced number.
    ///
    /// If a function is specified, it is passed the referenced heading and
    /// should return content.
    ///
    /// ```example
    /// #set heading(numbering: "1.", supplement: [Chapter])
    ///
    /// = Introduction <intro>
    /// In @intro, we see how to turn
    /// Sections into Chapters. And
    /// in @intro[Part], it is done
    /// manually.
    /// ```
    pub supplement: Smart<Option<Supplement>>,

    /// Whether the heading should appear in the outline.
    ///
    /// ```example
    /// #outline()
    ///
    /// #heading[Normal]
    /// This is a normal heading.
    ///
    /// #heading(outlined: false)[Hidden]
    /// This heading does not appear
    /// in the outline.
    /// ```
    #[default(true)]
    pub outlined: bool,

    /// Determines how to display the heading.
    ///
    /// ```example
    /// #set heading(
    ///   numbering: "1.1",
    ///   display: (numbering, content) =>
    ///     text(style: "italic", numbering) + h(1.5em, weak: true) + text(blue, content)
    /// )
    ///
    /// = A Heading
    /// == A Subheading
    ///
    /// #heading(numbering: none)[No Numbering]
    /// ```
    pub display: Option<Func>,

    /// Determines how to display the heading in an outline.
    ///
    /// ```example
    /// #outline()
    ///
    /// #set heading(
    ///   numbering: "1.1",
    ///   outline: (numbering, content) =>
    ///     text(style: "italic", numbering) + h(1.5em, weak: true) + text(blue, content)
    /// )
    ///
    /// = A Heading
    /// == A Subheading
    ///
    /// #heading(numbering: none)[No Numbering]
    /// ```
    pub outline: Option<Func>,

    /// The heading's title.
    #[required]
    pub body: Content,
}

impl Synthesize for HeadingElem {
    fn synthesize(&mut self, vt: &mut Vt, styles: StyleChain) -> SourceResult<()> {
        // Resolve the supplement.
        let supplement = match self.supplement(styles) {
            Smart::Auto => TextElem::packed(self.local_name_in(styles)),
            Smart::Custom(None) => Content::empty(),
            Smart::Custom(Some(supplement)) => supplement.resolve(vt, [self.clone()])?,
        };

        self.push_level(self.level(styles));
        self.push_numbering(self.numbering(styles));
        self.push_supplement(Smart::Custom(Some(Supplement::Content(supplement))));
        self.push_outlined(self.outlined(styles));
        self.push_display(self.display(styles));
        self.push_outline(self.outline(styles));

        Ok(())
    }
}

impl Show for HeadingElem {
    #[tracing::instrument(name = "HeadingElem::show", skip_all)]
    fn show(&self, vt: &mut Vt, styles: StyleChain) -> SourceResult<Content> {
        let body = self.body();
        let numbers = self.numbering(styles).map(|numbering| {
            Counter::of(Self::func())
                .display(Some(numbering), false)
                .spanned(self.span())
        });
        let display = self.display(styles);

        let realized = match (numbers, display) {
            (Some(numbers), None) => {
                numbers + HElem::new(Em::new(0.3).into()).with_weak(true).pack() + body
            }
            (Some(numbers), Some(display)) => display
                .call_vt(vt, [Value::Content(numbers), Value::Content(body)])?
                .display(),
            (None, Some(outline)) => {
                outline.call_vt(vt, [Value::None, Value::Content(body)])?.display()
            }
            (None, None) => self.body(),
        };

        Ok(BlockElem::new().with_body(Some(realized)).pack())
    }
}

impl Finalize for HeadingElem {
    fn finalize(&self, realized: Content, styles: StyleChain) -> Content {
        let level = self.level(styles).get();
        let scale = match level {
            1 => 1.4,
            2 => 1.2,
            _ => 1.0,
        };

        let size = Em::new(scale);
        let above = Em::new(if level == 1 { 1.8 } else { 1.44 }) / scale;
        let below = Em::new(0.75) / scale;

        let mut styles = Styles::new();
        styles.set(TextElem::set_size(TextSize(size.into())));
        styles.set(TextElem::set_weight(FontWeight::BOLD));
        styles.set(BlockElem::set_above(VElem::block_around(above.into())));
        styles.set(BlockElem::set_below(VElem::block_around(below.into())));
        styles.set(BlockElem::set_sticky(true));
        realized.styled_with_map(styles)
    }
}

impl Count for HeadingElem {
    fn update(&self) -> Option<CounterUpdate> {
        self.numbering(StyleChain::default())
            .is_some()
            .then(|| CounterUpdate::Step(self.level(StyleChain::default())))
    }
}

cast! {
    HeadingElem,
    v: Content => v.to::<Self>().ok_or("expected heading")?.clone(),
}

impl Refable for HeadingElem {
    fn supplement(&self) -> Content {
        // After synthesis, this should always be custom content.
        match self.supplement(StyleChain::default()) {
            Smart::Custom(Some(Supplement::Content(content))) => content,
            _ => Content::empty(),
        }
    }

    fn counter(&self) -> Counter {
        Counter::of(Self::func())
    }

    fn numbering(&self) -> Option<Numbering> {
        self.numbering(StyleChain::default())
    }
}

impl Outlinable for HeadingElem {
    fn outline(&self, vt: &mut Vt) -> SourceResult<Option<Content>> {
        let styles = StyleChain::default();
        if !self.outlined(styles) {
            return Ok(None);
        }

        let numbers = self
            .numbering(styles)
            .map(|numbering| {
                Counter::of(Self::func())
                    .at(vt, self.0.location().unwrap())?
                    .display(vt, &numbering)
            })
            .transpose()?;
        let body = self.body();
        let outline = self.outline(styles);

        let content = match (numbers, outline) {
            (Some(numbers), None) => numbers + SpaceElem::new().pack() + body,
            (Some(numbers), Some(outline)) => outline
                .call_vt(vt, [Value::Content(numbers), Value::Content(body)])?
                .display(),
            (None, Some(outline)) => {
                outline.call_vt(vt, [Value::None, Value::Content(body)])?.display()
            }
            (None, None) => self.body(),
        };

        Ok(Some(content))
    }

    fn level(&self) -> NonZeroUsize {
        self.level(StyleChain::default())
    }
}

impl LocalName for HeadingElem {
    fn local_name(&self, lang: Lang, region: Option<Region>) -> &'static str {
        match lang {
            Lang::ARABIC => "الفصل",
            Lang::BOKMÅL => "Kapittel",
            Lang::CHINESE if option_eq(region, "TW") => "小節",
            Lang::CHINESE => "小节",
            Lang::CZECH => "Kapitola",
            Lang::DANISH => "Afsnit",
            Lang::DUTCH => "Hoofdstuk",
            Lang::FRENCH => "Chapitre",
            Lang::GERMAN => "Abschnitt",
            Lang::ITALIAN => "Sezione",
            Lang::NYNORSK => "Kapittel",
            Lang::POLISH => "Sekcja",
            Lang::PORTUGUESE => "Seção",
            Lang::RUSSIAN => "Раздел",
            Lang::SLOVENIAN => "Poglavje",
            Lang::SPANISH => "Sección",
            Lang::SWEDISH => "Kapitel",
            Lang::UKRAINIAN => "Розділ",
            Lang::VIETNAMESE => "Phần", // TODO: This may be wrong.
            Lang::ENGLISH | _ => "Section",
        }
    }
}
