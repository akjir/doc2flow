import type { Core } from './core/core.ts';
import type { Export } from './core/export.ts';
import type { Language } from './core/lang.ts';
import type { Storage } from './core/storage.ts';
import type { Utils } from './core/utils.ts';

declare global {
    interface Window {
        d2f: {
            core: Core;
            document: {
                id: String;
            };
            storage: Storage;
            export: Export;
            utils: Utils;
            lang: Language;
        }
    }
}