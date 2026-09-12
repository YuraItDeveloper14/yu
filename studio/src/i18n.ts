// Everything the page says, in Ukrainian and English.
import type { YuLang } from './bridge.ts';

const UK = {
  run: 'Запустити',
  stop: 'Стоп',
  runHint: 'Ctrl+Enter',
  share: 'Поділитися',
  examples: 'Приклади',
  picture: 'Малюнок',
  output: 'Вивід',
  theme: 'Змінити тему',
  otherLang: 'EN',
  otherLangName: 'English',
  loading: 'Завантажую Yu…',
  done: 'Готово',
  stopped: 'Зупинено',
  copied: 'Посилання скопійовано',
  badLink: 'Посилання пошкоджене, тому відкрито останню програму',
  empty: 'Тут з’явиться малюнок. Відкрий приклад або напиши коло(300, 200, 80) і натисни «Запустити».',
  nothingDrawn: 'Ця програма нічого не намалювала.',
  answer: 'Відповідь',
  noInput: 'Відповісти на запитай тут не вийде: браузер не дав спільної пам’яті.',
  crash: 'Внутрішня помилка Yu',
  saveSvg: 'SVG',
  savePng: 'PNG',
  saveTitle: 'Зберегти малюнок',
  fileName: 'малюнок',
  noPicture: 'Спершу щось намалюй',
  linkInBar: 'Посилання вже в адресному рядку',
};

const EN: typeof UK = {
  noPicture: 'Draw something first',
  linkInBar: 'The link is in the address bar',
  run: 'Run',
  stop: 'Stop',
  runHint: 'Ctrl+Enter',
  share: 'Share',
  examples: 'Examples',
  picture: 'Picture',
  output: 'Output',
  theme: 'Switch theme',
  otherLang: 'UA',
  otherLangName: 'Українська',
  loading: 'Loading Yu…',
  done: 'Done',
  stopped: 'Stopped',
  copied: 'Link copied',
  badLink: 'The link is broken, so your last program is open',
  empty: 'Your picture appears here. Open an example or write circle(300, 200, 80) and press Run.',
  nothingDrawn: "This program didn't draw anything.",
  answer: 'Answer',
  noInput: "Answers to ask can't work here: the browser gave no shared memory.",
  crash: 'Internal Yu error',
  saveSvg: 'SVG',
  savePng: 'PNG',
  saveTitle: 'Save the picture',
  fileName: 'picture',
};

export type Key = keyof typeof UK;

export function text(lang: YuLang, key: Key): string {
  return (lang === 'uk' ? UK : EN)[key];
}

/** The examples menu, in order; `id` is the file name in examples/. */
export const EXAMPLES: { id: string; uk: string; en: string }[] = [
  { id: 'star', uk: 'Зірка', en: 'Star' },
  { id: 'house', uk: 'Будинок', en: 'House' },
  { id: 'sun', uk: 'Сонце', en: 'Sun' },
  { id: 'spiral', uk: 'Спіраль', en: 'Spiral' },
  { id: 'flag', uk: 'Прапор', en: 'Flag' },
  { id: 'hello', uk: 'Привіт', en: 'Hello' },
  { id: 'fizzbuzz', uk: 'FizzBuzz', en: 'FizzBuzz' },
  { id: 'guess', uk: 'Вгадай число', en: 'Guess the number' },
  { id: 'functions', uk: 'Функції', en: 'Functions' },
  { id: 'lists', uk: 'Списки', en: 'Lists' },
  { id: 'typo', uk: 'Помилка з підказкою', en: 'A friendly error' },
];
