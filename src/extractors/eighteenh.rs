use super::*;

def_regex2![
    URL  => r#"(https?://18h.animezilla.com/manga/\d+)"#,
    LAST => r#"https?://18h.animezilla.com/manga/\d+/(\d+)"#
];

/// 对 18h.animezilla.com 内容的抓取实现
def_extractor! {
    state	=> [usable: true, pageable: true, searchable: false],
    tags	=> [Chinese, NSFW],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://18h.animezilla.com/manga/page/{}", page);

        itemsgen2!(
            url         = &url,
            parent_dom  = r#".pure-u-1-2 > article[id^="post-"]"#,
            cover_dom   = "a img",
            link_dom    = ".entry-title > a"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        comic.push_chapter(Chapter::from(&*comic));

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        chapter.url = match_content2!(&chapter.url, &*URL_RE)?.to_string();
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.title = document.dom_attr(r#"meta[itemprop="name"]"#, "content")?;
        let last_url = document.dom_attr("a.last", "href")?;
        let total = match_content2!(&last_url, &*LAST_RE)?.parse::<i32>()?;

        let url = chapter.url.clone();
        let fetch = Box::new(move |current_page: usize| {
            let page_url = format!("{}/{}", url, current_page);
            let page_html = get(&page_url)?.text()?;
            let page_document = parse_document(&page_html);
            let address = page_document.dom_attr("img#comic", "src")?;
            Ok(vec![Page::new(current_page - 1, address)])
        });

        Ok(ChapterPages::new(chapter, total, vec![], fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(48, comics.len());
        let chapter1 = &mut Chapter::from_link("", "https://18h.animezilla.com/manga/2940");
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!(
            "[中文同人A漫][YU-RI] 成長しました。/現已成長。 (海賊王) [17P]",
            chapter1.title
        );
        assert_eq!(17, chapter1.pages.len());
    }
}
