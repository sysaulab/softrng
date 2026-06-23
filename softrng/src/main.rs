use minus::Pager;
use termimad::MadSkin;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let markdown = include_str!("../assets/manual.txt");

    let skin = MadSkin::default();
    // term_text returns a FmtText that implements Display – convert it to String
    let formatted = format!("{}", skin.term_text(markdown));

    let pager = Pager::new();
    pager.set_text(formatted)?;
    // dynamic_paging is always available (page_all requires feature "static_output")
    minus::dynamic_paging(pager)?;

    Ok(())
}
