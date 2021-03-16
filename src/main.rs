#[macro_use]
extern crate concat_with;
extern crate clap;
extern crate terminal_size;

extern crate path_absolutize;

extern crate comrak;
extern crate html_escape;
extern crate html_minifier;

#[macro_use]
extern crate slash_formatter;

#[macro_use]
extern crate lazy_static_include;

use std::borrow::Cow;
use std::error::Error;
use std::fs;
use std::path::Path;

use clap::{App, Arg};
use terminal_size::terminal_size;

use path_absolutize::Absolutize;

use comrak::{markdown_to_html, ComrakOptions};
use html_minifier::HTMLMinifier;

const APP_NAME: &str = "Markdown to HTML Converter";
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const CARGO_PKG_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

lazy_static_include_str! {
    MARKDOWN_CSS => concat_with_file_separator!("resources", "github-markdown.css"),
    FONT_CJK => concat_with_file_separator!("resources", "font-cjk.css"),
    FONT_CJK_MONO => concat_with_file_separator!("resources", "font-cjk-mono.css"),
    GITHUB => concat_with_file_separator!("resources", "github.css"),
    WEBFONT => concat_with_file_separator!("resources", "webfont.js"),
    HIGHLIGHT_CODE => concat_with_file_separator!("resources", "highlight-code.js"),
    MATH_JAX => concat_with_file_separator!("resources", "mathjax.min.js"),
    MATH_JAX_CONFIG => concat_with_file_separator!("resources", "mathjax-config.js"),
    HIGHLIGHT => concat_with_file_separator!("resources", "highlight.pack.min.js"),
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new(APP_NAME)
        .set_term_width(terminal_size().map(|(width, _)| width.0 as usize).unwrap_or(0))
        .version(CARGO_PKG_VERSION)
        .author(CARGO_PKG_AUTHORS)
        .about(concat!("A simple tool for converting Simple Chinese to Traditional Chinese(TW).\n\nEXAMPLES:\n", concat_line!(prefix "ef ",
            "/path/to/file.md                            # Convert /path/to/file.md to /path/to/file.html, titled \"file\"",
            "/path/to/file.md -o /path/to/output.html    # Convert /path/to/file.md to /path/to/output.html, titled \"output\"",
            "/path/to/file.md -t 'Hello World!'          # Convert /path/to/file.md to /path/to/file.html, titled \"Hello World!\"",
        )))
        .arg(Arg::with_name("TITLE")
            .required(false)
            .long("title")
            .short("t")
            .help("Specifies the title of your HTML file")
            .takes_value(true)
            .display_order(1)
        )
        .arg(Arg::with_name("MARKDOWN_PATH")
            .required(true)
            .help("Specifies the path of your Markdown file")
            .takes_value(true)
        )
        .arg(Arg::with_name("HTML_PATH")
            .required(false)
            .long("html-path")
            .short("o")
            .help("Specifies the path of your HTML file")
            .takes_value(true)
            .display_order(2)
        )
        .arg(Arg::with_name("FORCE")
            .long("force")
            .short("f")
            .help("Forces to output if the HTML file exists")
        )
        .arg(Arg::with_name("NO_SAFE")
            .required(false)
            .long("no-safe")
            .help("Allows raw HTML and dangerous URLs")
            .display_order(3)
        )
        .arg(Arg::with_name("NO_HIGHLIGHT")
            .required(false)
            .long("no-highlight")
            .help("Not allow to use highlight.js")
            .display_order(4)
        )
        .arg(Arg::with_name("NO_MATHJAX")
            .required(false)
            .long("no-mathjax")
            .help("Not allow to use mathjax.js")
            .display_order(5)
        )
        .arg(Arg::with_name("NO_CJK_FONTS")
            .required(false)
            .long("no-cjk-fonts")
            .help("Not allow to use CJK fonts")
            .display_order(6)
        )
        .arg(Arg::with_name("CSS_PATH")
            .required(false)
            .long("css-path")
            .help("Specifies the path of your custom CSS file")
            .takes_value(true)
            .display_order(100)
        )
        .arg(Arg::with_name("HIGHLIGHT_JS_PATH")
            .required(false)
            .long("highlight-js-path")
            .help("Specifies the path of your custom highlight.js file")
            .takes_value(true)
            .display_order(101)
        )
        .arg(Arg::with_name("HIGHLIGHT_CSS_PATH")
            .required(false)
            .long("highlight-css-path")
            .help("Specifies the path of your custom CSS file for highlight.js code blocks")
            .takes_value(true)
            .display_order(102)
        )
        .arg(Arg::with_name("MATHJAX_JS_PATH")
            .required(false)
            .long("mathjax-path-path")
            .help("Specifies the path of your custom single MATH_JAX.js file")
            .takes_value(true)
            .display_order(103)
        )
        .after_help("Enjoy it! https://magiclen.org")
        .get_matches();

    let title = matches.value_of("TITLE");
    let markdown_path = matches.value_of("MARKDOWN_PATH").unwrap();
    let html_path = matches.value_of("HTML_PATH");

    let force = matches.is_present("FORCE");
    let no_safe = matches.is_present("NO_SAFE");
    let no_highlight = matches.is_present("NO_HIGHLIGHT");
    let no_mathjax = matches.is_present("NO_MATHJAX");
    let no_cjk_fonts = matches.is_present("NO_CJK_FONTS");

    let css_path = matches.value_of("CSS_PATH");
    let highlight_js_path = matches.value_of("HIGHLIGHT_JS_PATH");
    let highlight_css_path = matches.value_of("HIGHLIGHT_CSS_PATH");
    let mathjax_js_path = matches.value_of("MATHJAX_JS_PATH");

    let markdown_path = Path::new(markdown_path);

    if markdown_path.is_dir() {
        return Err(
            format!("`{}` is a directory!", markdown_path.absolutize()?.to_string_lossy()).into()
        );
    }

    let file_ext =
        markdown_path.extension().map(|ext| ext.to_string_lossy()).unwrap_or_else(|| "".into());

    if !file_ext.eq_ignore_ascii_case("md") && !file_ext.eq_ignore_ascii_case("markdown") {
        return Err(format!(
            "`{}` is not a Markdown file.",
            markdown_path.absolutize()?.to_string_lossy()
        )
        .into());
    }

    let file_stem =
        markdown_path.file_stem().map(|ext| ext.to_string_lossy()).unwrap_or_else(|| "".into());

    let html_path = match html_path {
        Some(html_path) => Cow::from(Path::new(html_path)),
        None => {
            let folder_path = markdown_path.parent().unwrap();

            Cow::from(folder_path.join(format!("{}.html", file_stem)))
        }
    };

    if let Ok(metadata) = html_path.metadata() {
        if metadata.is_dir() || !force {
            return Err(format!("`{}` exists!", html_path.absolutize()?.to_string_lossy()).into());
        }
    }

    let title = match title {
        Some(title) => Cow::from(title),
        None => file_stem,
    };

    let markdown = fs::read_to_string(markdown_path)?;

    let markdown_html = {
        let mut options = ComrakOptions::default();

        if no_safe {
            options.render.unsafe_ = true;
        }

        options.extension.autolink = true;
        options.extension.description_lists = true;
        options.extension.footnotes = true;
        options.extension.strikethrough = true;
        options.extension.superscript = true;
        options.extension.table = true;
        options.extension.tagfilter = true;
        options.extension.tasklist = true;
        options.render.hardbreaks = true;

        markdown_to_html(&markdown, &options)
    };

    let mut html_minifier = HTMLMinifier::new();

    html_minifier.digest("<!DOCTYPE html>")?;
    html_minifier.digest("<html>")?;

    html_minifier.digest("<head>")?;
    html_minifier.digest("<meta charset=UTF-8>")?;
    html_minifier.digest("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1, shrink-to-fit=no\">")?;
    html_minifier.digest(format!(
        "<meta name=\"generator\" content=\"{} {} by magiclen.org\"/>",
        APP_NAME, CARGO_PKG_VERSION,
    ))?;
    html_minifier.digest("<title>")?;
    html_minifier.digest(html_escape::encode_text(title.as_ref()).as_ref())?;
    html_minifier.digest("</title>")?;

    html_minifier.digest("<style>")?;
    match css_path {
        Some(with_css_path) => {
            let with_css = fs::read_to_string(with_css_path)?;

            html_minifier.digest(html_escape::encode_style(&with_css).as_ref())?;
        }
        None => {
            html_minifier.digest(*MARKDOWN_CSS)?;
        }
    }
    html_minifier.digest("</style>")?;

    let has_code = {
        if no_highlight {
            false
        } else {
            markdown_html.contains("</code></pre>")
        }
    };

    let has_mathjax = {
        if no_mathjax {
            false
        } else {
            markdown_html.contains("#{{")
        }
    };

    if !no_cjk_fonts {
        html_minifier.digest("<style>")?;
        html_minifier.digest(*FONT_CJK)?;
        html_minifier.digest("</style>")?;

        html_minifier.digest("<style>")?;
        html_minifier.digest(*FONT_CJK_MONO)?;
        html_minifier.digest("</style>")?;
    }

    if has_code {
        html_minifier.digest("<script>")?;
        match highlight_js_path {
            Some(with_highlight_js_path) => {
                let with_highlight_js = fs::read_to_string(with_highlight_js_path)?;
                html_minifier.digest(html_escape::encode_script(&with_highlight_js).as_ref())?;
            }
            None => unsafe {
                html_minifier.indigest(*HIGHLIGHT);
            },
        }
        html_minifier.digest("</script>")?;

        html_minifier.digest("<style>")?;
        match highlight_css_path {
            Some(with_highlight_css_path) => {
                let with_highlight_css = fs::read_to_string(with_highlight_css_path)?;

                html_minifier.digest(html_escape::encode_style(&with_highlight_css).as_ref())?;
            }
            None => {
                html_minifier.digest(*GITHUB)?;
            }
        }
        html_minifier.digest("</style>")?;
    }

    if has_mathjax {
        html_minifier.digest("<script>")?;
        html_minifier.digest(*MATH_JAX_CONFIG)?;
        html_minifier.digest("</script>")?;

        html_minifier.digest("<script>")?;
        match mathjax_js_path {
            Some(with_mathjax_js_path) => {
                let with_mathjax_js = fs::read_to_string(with_mathjax_js_path)?;
                html_minifier.digest(html_escape::encode_script(&with_mathjax_js).as_ref())?;
            }
            None => unsafe {
                html_minifier.indigest(*MATH_JAX);
            },
        }
        html_minifier.digest("</script>")?;
    }

    html_minifier.digest("</head>")?;

    html_minifier.digest("<body>")?;

    html_minifier.digest("<article class=\"markdown-body\">")?;
    html_minifier.digest(&markdown_html)?;
    html_minifier.digest("</article>")?;

    if !no_cjk_fonts {
        html_minifier.digest("<script>")?;
        html_minifier.digest(*WEBFONT)?;
        html_minifier.digest("</script>")?;
    }

    if has_code {
        html_minifier.digest("<script>")?;
        html_minifier.digest(*HIGHLIGHT_CODE)?;
        html_minifier.digest("</script>")?;
    }

    html_minifier.digest("</body>")?;

    html_minifier.digest("</html>")?;

    let minified_html = html_minifier.get_html();

    fs::write(html_path, minified_html)?;

    Ok(())
}
