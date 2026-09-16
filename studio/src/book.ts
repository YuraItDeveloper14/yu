// The book's pages: the theme they share with the Studio, the language they remember, the search.
import '@fontsource-variable/inter';
import '@fontsource-variable/jetbrains-mono';
import './theme.css';
import './book.css';

interface Found {
  title: string;
  url: string;
  headings: { text: string; anchor: string }[];
}

/** localStorage may be missing or throw (private windows); the page works without it. */
const store = {
  get(key: string): string | null {
    try {
      return localStorage.getItem(key);
    } catch {
      return null;
    }
  },
  set(key: string, value: string): void {
    try {
      localStorage.setItem(key, value);
    } catch {
      // Not remembered this time.
    }
  },
};

const lang = document.body.dataset.lang === 'en' ? 'en' : 'uk';

document.getElementById('book-theme')?.addEventListener('click', () => {
  const next = document.documentElement.dataset.theme === 'light' ? 'dark' : 'light';
  document.documentElement.dataset.theme = next;
  store.set('yu-theme', next);
});

// Reading the book in one language means the Studio opens in it too.
document.getElementById('book-other')?.addEventListener('click', (event) => {
  const other = (event.currentTarget as HTMLElement).dataset.lang;
  if (other) store.set('yu-lang', other);
});

const search = document.getElementById('book-search') as HTMLInputElement | null;
const results = document.getElementById('book-results');

if (search && results) {
  let chapters: Found[] = [];
  const load = fetch(`/book/search-${lang}.json`)
    .then((response) => response.json() as Promise<Found[]>)
    .then((found) => {
      chapters = found;
    })
    .catch(() => {
      chapters = [];
    });

  const show = () => {
    const query = search.value.trim().toLowerCase();
    if (!query) {
      results.hidden = true;
      results.replaceChildren();
      return;
    }
    const hits: { text: string; url: string }[] = [];
    for (const chapter of chapters) {
      if (chapter.title.toLowerCase().includes(query)) hits.push({ text: chapter.title, url: chapter.url });
      for (const heading of chapter.headings) {
        if (heading.text.toLowerCase().includes(query)) {
          hits.push({ text: `${chapter.title} · ${heading.text}`, url: `${chapter.url}#${heading.anchor}` });
        }
      }
    }
    results.replaceChildren(
      ...hits.slice(0, 8).map((hit) => {
        const link = document.createElement('a');
        link.href = hit.url;
        link.textContent = hit.text;
        return link;
      }),
    );
    if (!hits.length) {
      const empty = document.createElement('p');
      empty.className = 'none';
      empty.textContent = lang === 'uk' ? 'Нічого не знайшлося' : 'Nothing found';
      results.append(empty);
    }
    results.hidden = false;
  };

  search.addEventListener('input', () => void load.then(show));
  search.addEventListener('keydown', (event) => {
    if (event.key !== 'Enter') return;
    const first = results.querySelector('a');
    if (first) location.href = first.href;
  });
  document.addEventListener('click', (event) => {
    if (!results.contains(event.target as Node) && event.target !== search) results.hidden = true;
  });
}
