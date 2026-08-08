export type ExportType = 'PDF' | 'DOCUMENT';
export type ExportHandler = (type: ExportType) => void;

export interface Export {
    export(type: ExportType): void;
    registerExportHandler(handler: ExportHandler): void;
}

const exportHandlers = new Set<ExportHandler>();

window.d2f = window.d2f || {};
window.d2f.export = {
    export: performExport,
    registerExportHandler,
};

function registerExportHandler(handler: ExportHandler): void {
    exportHandlers.add(handler);
}

function performExport(type: ExportType): void {
    for (const handler of exportHandlers) {
        try {
            handler(type);
        } catch (e) {
            console.warn('Failed to execute export handler', e);
        }
    }

    if (type === 'PDF') {
        const collapsed = Array.from(document.querySelectorAll<HTMLElement>('.sb.collapsed'));
        collapsed.forEach((el) => el.classList.remove('collapsed'));

        const restore = (): void => {
            collapsed.forEach((el) => el.classList.add('collapsed'));
            window.removeEventListener('afterprint', restore);
        };

        window.addEventListener('afterprint', restore);
        setTimeout(() => window.print(), 100);
        return;
    }

    if (type === 'DOCUMENT') {
        window.d2f.storage.saveState();

        document.querySelectorAll<HTMLInputElement>('input.persistent-field, input[type="text"]').forEach((input) => {
            input.setAttribute('value', input.value);
        });
        document.querySelectorAll<HTMLInputElement>('input[type="checkbox"]').forEach((cb) => {
            if (cb.checked) {
                cb.setAttribute('checked', '');
            } else {
                cb.removeAttribute('checked');
            }
        });
        document.querySelectorAll<HTMLTextAreaElement>('textarea').forEach((ta) => {
            ta.textContent = ta.value;
            ta.setAttribute('value', ta.value);
        });

        const rawFilename = window.location.pathname.split('/').pop() || 'index.html';
        const filename = decodeURIComponent(rawFilename || 'index.html');

        const htmlContent = '<!DOCTYPE html>\n' + document.documentElement.outerHTML;
        const blob = new Blob([htmlContent], { type: 'text/html;charset=utf-8' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }
}