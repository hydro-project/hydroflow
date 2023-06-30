import siteConfig from '@generated/docusaurus.config';
export default function prismIncludeLanguages(PrismObject) {
  const {
    themeConfig: {prism},
  } = siteConfig;
  const {additionalLanguages} = prism;
  // Prism components work on the Prism instance on the window, while prism-
  // react-renderer uses its own Prism instance. We temporarily mount the
  // instance onto window, import components to enhance it, then remove it to
  // avoid polluting global namespace.
  // You can mutate PrismObject: registering plugins, deleting languages... As
  // long as you don't re-assign it
  globalThis.Prism = PrismObject;
  additionalLanguages.forEach((lang) => {
    // eslint-disable-next-line global-require, import/no-dynamic-require
    require(`prismjs/components/prism-${lang}`);
  });
  const rustLanguage = Prism.languages.rust;
  Prism.languages["rust,ignore"] = Prism.languages.rust;

  const origTokenize = PrismObject.tokenize;
  PrismObject.tokenize = (text, grammar) => {
    if (grammar == rustLanguage) {
      text = text.split("\n").filter(line => !line.startsWith("# ")).join("\n");
    }
    return origTokenize(text, grammar);
  };

  delete globalThis.Prism;
}
