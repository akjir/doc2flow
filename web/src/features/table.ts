function initSectionTables(): void {
    const wrappers = document.querySelectorAll<HTMLElement>('.item-table-wrap, .section table');
    let tableCount = 0;

    wrappers.forEach((wrapper) => {
        const table = wrapper instanceof HTMLTableElement
            ? wrapper
            : wrapper.querySelector<HTMLTableElement>('table');

        if (!table) return;
        tableCount += 1;

        if (!table.classList.contains('item-table')) {
            table.classList.add('item-table');
        }

        const tbody = table.querySelector<HTMLTableSectionElement>('tbody');
        if (tbody && !tbody.hasAttribute('data-hover-bound')) {
            tbody.setAttribute('data-hover-bound', 'true');

            tbody.addEventListener('mouseover', (event: MouseEvent) => {
                const target = event.target;
                if (target instanceof Element) {
                    const row = target.closest<HTMLTableRowElement>('tr');
                    if (row && row.parentElement === tbody) {
                        row.classList.add('row-hover');
                    }
                }
            });

            tbody.addEventListener('mouseout', (event: MouseEvent) => {
                const target = event.target;
                if (target instanceof Element) {
                    const row = target.closest<HTMLTableRowElement>('tr');
                    if (row && row.parentElement === tbody) {
                        row.classList.remove('row-hover');
                    }
                }
            });
        }
    });
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        initSectionTables();
    });
} else {
    initSectionTables();
}
