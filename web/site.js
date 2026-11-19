// The home page's language: ?lang=xx, then the one picked before, then the browser's, then
// English (written in index.html). i18n/<lang>.json maps the data-i18n keys to the text.
(() => {
  const langs = ["en", "ru", "lt", "be"];
  const select = document.getElementById("lang");
  const english = new Map();
  const attrs = { "data-i18n": null, "data-i18n-alt": "alt", "data-i18n-aria": "aria-label" };
  for (const [attr, prop] of Object.entries(attrs)) {
    for (const el of document.querySelectorAll(`[${attr}]`)) {
      english.set(el, { ...english.get(el), [attr]: prop ? el.getAttribute(prop) : el.textContent });
    }
  }
  const englishTitle = document.title;

  function stored() {
    try { return localStorage.getItem("lang"); } catch { return null; }
  }
  function pick() {
    const asked = new URLSearchParams(location.search).get("lang");
    for (const l of [asked, stored(), ...(navigator.languages ?? [navigator.language])]) {
      const short = (l ?? "").slice(0, 2).toLowerCase();
      if (langs.includes(short)) return short;
    }
    return "en";
  }
  async function apply(lang) {
    let strings = {};
    if (lang !== "en") {
      try { strings = await (await fetch(`i18n/${lang}.json`)).json(); } catch { lang = "en"; }
    }
    for (const [el, orig] of english) {
      for (const [attr, text] of Object.entries(orig)) {
        const value = strings[el.getAttribute(attr)] ?? text;
        const prop = attrs[attr];
        if (prop) el.setAttribute(prop, value); else el.textContent = value;
      }
    }
    document.title = strings["page-title"] ?? englishTitle;
    document.documentElement.lang = lang;
    select.value = lang;
  }
  select.addEventListener("change", () => {
    try { localStorage.setItem("lang", select.value); } catch {}
    apply(select.value);
  });
  apply(pick());
})();
