import siteConfig from '@generated/docusaurus.config';
export default function prismIncludeLanguages(PrismObject) {
  const {
    themeConfig: {prism},
  } = siteConfig;
  const {additionalLanguages} = prism;
  
  const PrismBefore = globalThis.Prism;
  globalThis.Prism = PrismObject;
  additionalLanguages.forEach((lang) => {
    // eslint-disable-next-line global-require, import/no-dynamic-require
    require(`prismjs/components/prism-${lang}`);
  });
  Prism.languages["rust,ignore"] = Prism.languages.rust;

  const origTokenize = PrismObject.tokenize;
  PrismObject.hooks.add("after-tokenize", function(env) {
    if (env.language == "rust") {
      let code = env.code.split("\n").filter(line => !line.startsWith("# ")).join("\n");
      env.tokens = origTokenize(code, env.grammar);
    }
  });

  delete globalThis.Prism;
  if (typeof PrismBefore !== 'undefined') {
    globalThis.Prism = PrismObject;
  }
}
