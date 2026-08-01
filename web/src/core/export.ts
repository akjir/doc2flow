export interface Export {
    exportPDF(): void;
    saveDocumentState(): void;
}

function exportPDF(): void {
    const collapsed = Array.from(document.querySelectorAll<HTMLElement>('.sb.collapsed'));
    collapsed.forEach((el) => el.classList.remove('collapsed'));

    const restore = (): void => {
        collapsed.forEach((el) => el.classList.add('collapsed'));
        window.removeEventListener('afterprint', restore);
    };

    window.addEventListener('afterprint', restore);
    setTimeout(() => window.print(), 100);
}

function saveDocumentState(): void {
    window.d2f.storage.saveState();

    const checkboxes = document.querySelectorAll<HTMLInputElement>('.check-item input[type="checkbox"]');
    checkboxes.forEach((cb) => {
        if (cb.checked) {
            cb.setAttribute('checked', 'checked');
        } else {
            cb.removeAttribute('checked');
        }
        window.d2f.tasks?.styleItem(cb);
    });

    const inputs = document.querySelectorAll<HTMLInputElement>('input.persistent-field, .info-table input');
    inputs.forEach((input) => {
        input.setAttribute('value', input.value);
    });

    const textareas = document.querySelectorAll<HTMLTextAreaElement>('textarea.item-comment-input');
    textareas.forEach((ta) => {
        ta.textContent = ta.value;
        ta.setAttribute('value', ta.value);
    });

    const rawFilename = window.location.pathname.split('/').pop() ?? 'index.html';
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

window.d2f.export = {
    exportPDF,
    saveDocumentState,
};

