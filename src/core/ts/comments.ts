export interface CommentBoxResult {
    readonly box: HTMLElement;
    readonly input: HTMLTextAreaElement;
}

function autoExpandTextarea(el: HTMLTextAreaElement | null): void {
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = String(el.scrollHeight) + 'px';
}

function getOrCreateCommentBox(checkItem: HTMLElement | null, initialValue?: string): CommentBoxResult | null {
    if (!checkItem) return null;
    let box = checkItem.querySelector<HTMLElement>('.item-comment-box');
    let input: HTMLTextAreaElement | null = null;

    if (!box) {
        box = document.createElement('div');
        box.className = 'item-comment-box';

        input = document.createElement('textarea');
        input.rows = 1;
        input.className = 'item-comment-input';
        const i18n = window.d2f.lang.dictionary;
        const commentLabel = i18n.comment_placeholder ?? 'Add a comment...';
        input.placeholder = commentLabel;
        input.setAttribute('aria-label', commentLabel);

        const delBtn = document.createElement('button');
        delBtn.type = 'button';
        delBtn.className = 'item-comment-del';
        delBtn.title = 'Delete comment';
        delBtn.setAttribute('aria-label', 'Delete comment');
        delBtn.innerHTML = '&#10006;';

        box.appendChild(input);
        box.appendChild(delBtn);
        checkItem.appendChild(box);
    } else {
        const rawInput = box.querySelector('.item-comment-input');
        input = rawInput instanceof HTMLTextAreaElement ? rawInput : null;
    }

    if (!input) return null;

    if (typeof initialValue === 'string') {
        input.value = initialValue;
        input.textContent = initialValue;
        input.setAttribute('value', initialValue);
    }

    autoExpandTextarea(input);
    return { box, input };
}

function saveComments(): Record<string, unknown> {
    const comments: Record<string, string> = {};
    document.querySelectorAll<HTMLElement>('.doc-item').forEach((item, index) => {
        const input = item.querySelector<HTMLTextAreaElement>('.item-comment-input');
        if (input && input.value.trim() !== '') {
            const key = item.id || ('item_' + String(index));
            comments[key] = input.value;
        }
    });
    return { comments };
}

function loadComments(state: Record<string, unknown>): boolean {
    const comments = state['comments'];
    if (window.d2f.utils.isRecord(comments)) {
        document.querySelectorAll<HTMLElement>('.doc-item').forEach((item, index) => {
            const key = item.id || ('item_' + String(index));
            const val = comments[key];
            if (val !== undefined && typeof val === 'string') {
                getOrCreateCommentBox(item, val);
            }
        });
    }
    return false;
}

function resetComments(): void {
    document.querySelectorAll<HTMLElement>('.item-comment-box').forEach((box) => {
        box.remove();
    });
}

if (typeof window !== 'undefined') {
    window.d2f.storage.registerSaveHandler(saveComments);
    window.d2f.storage.registerLoadHandler(loadComments);

    document.addEventListener('DOMContentLoaded', () => {
        window.d2f.core.registerResetHandler(resetComments);
    });

    const saveStateDebounced = window.d2f.utils.debounce(() => window.d2f.storage.saveState(), 300);

    document.addEventListener('click', (e: MouseEvent) => {
        const target = e.target;
        if (!(target instanceof Element)) return;

        const commentBtn = target.closest<HTMLElement>('.item-comment-icon');
        if (commentBtn) {
            const docItem = commentBtn.closest<HTMLElement>('.doc-item');
            if (docItem) {
                const res = getOrCreateCommentBox(docItem);
                if (res?.input) {
                    res.input.focus();
                }
            }
            return;
        }

        const commentDelBtn = target.closest<HTMLElement>('.item-comment-del');
        if (commentDelBtn) {
            const box = commentDelBtn.closest<HTMLElement>('.item-comment-box');
            if (box) {
                box.remove();
                window.d2f.storage.saveState();
            }
        }
    });

    const handleCommentInput = (e: Event): void => {
        const target = e.target;
        if (target instanceof HTMLTextAreaElement && target.classList.contains('item-comment-input')) {
            target.textContent = target.value;
            target.setAttribute('value', target.value);
            saveStateDebounced();
        }
    };

    document.addEventListener('input', handleCommentInput);
    document.addEventListener('change', handleCommentInput);
}