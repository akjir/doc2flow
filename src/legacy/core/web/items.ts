function saveItems(): Record<string, unknown> {
    const texts: Record<string, boolean> = {};
    document.querySelectorAll<HTMLElement>('.doc-item.text-item, .doc-item.simple-item').forEach((item, index) => {
        const key = item.id || ('txt_' + String(index));
        texts[key] = item.classList.contains('checked');
    });

    return { texts };
}

function loadItems(state: Record<string, unknown>): boolean {
    const textsData = state['texts'];
    if (window.d2f.utils.isRecord(textsData)) {
        document.querySelectorAll<HTMLElement>('.doc-item.text-item, .doc-item.simple-item').forEach((item, index) => {
            const key = item.id || ('txt_' + String(index));
            const val = textsData[key];
            if (typeof val === 'boolean') {
                item.classList.toggle('checked', val);
            }
        });
    }
    return false;
}

function resetItems(): void {
    document.querySelectorAll<HTMLElement>('.doc-item.text-item, .doc-item.simple-item').forEach((item) => {
        item.classList.remove('checked');
    });
}

if (typeof window !== 'undefined') {
    window.d2f.storage.registerSaveHandler(saveItems);
    window.d2f.storage.registerLoadHandler(loadItems);

    document.addEventListener('DOMContentLoaded', () => {
        window.d2f.core.registerResetHandler(resetItems);
    });

    document.addEventListener('click', (e: MouseEvent) => {
        const target = e.target;
        if (!(target instanceof Element)) return;

        const docItem = target.closest<HTMLElement>('.doc-item');
        if (docItem) {
            if (
                target.tagName === 'A' ||
                target.tagName === 'IMG' ||
                target.tagName === 'INPUT' ||
                target.closest('.item-comment-box') ||
                target.closest('.item-comment-icon') ||
                target.closest('.item-comment-del')
            ) {
                return;
            }

            if (docItem.classList.contains('text-item') || docItem.classList.contains('simple-item')) {
                docItem.classList.toggle('checked');
                window.d2f.storage.saveState();
            }
        }
    });
}