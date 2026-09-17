// /book/ leads to the first chapter in the reader's language: the Studio's choice, else the browser's.
let lang = null;
try {
  lang = localStorage.getItem('yu-lang');
} catch {
  // No storage in this window.
}
lang ??= navigator.language.slice(0, 2);
location.replace(lang === 'en' ? '/book/en/01-start/' : '/book/uk/01-start/');
