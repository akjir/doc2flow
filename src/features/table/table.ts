interface Window {
    d2f_table: { readonly init: () => void; };
}

(() => {
    function initSectionTables(): void {
        const wrappers = document.querySelectorAll<HTMLElement>('.item-table-wrap, .section table');

        wrappers.forEach((wrapper) => {
            const table = wrapper instanceof HTMLTableElement
                ? wrapper
                : wrapper.querySelector<HTMLTableElement>('table');

            if (!table) return;

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

    window.d2f_table = {
        init: initSectionTables,
    };

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initSectionTables);
    } else {
        initSectionTables();
    }
})();
