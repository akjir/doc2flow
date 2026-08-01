interface TocItem {
    readonly id: string;
    readonly text: string;
    readonly level: number;
}

function generateTocItems(): readonly TocItem[] {
    const headings = document.querySelectorAll<HTMLElement>('.sh');
    const items: TocItem[] = [];

    headings.forEach((sh, index) => {
        const section = sh.closest<HTMLElement>('.section');
        const id = section?.id || ('section_' + String(index + 1));
        const text = sh.textContent?.trim() ?? ('Section ' + String(index + 1));
        const isH1 = sh.classList.contains('sh-h1');
        items.push({
            id,
            text,
            level: isH1 ? 1 : 2
        });
    });

    return items;
}

function highlightActiveTocItem(activeId: string): void {
    const tocLinks = document.querySelectorAll<HTMLAnchorElement>('.toc-link');
    tocLinks.forEach((link) => {
        const isMatch = link.getAttribute('href') === '#' + activeId;
        link.classList.toggle('active', isMatch);
    });
}
