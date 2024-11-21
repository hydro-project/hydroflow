// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require('prism-react-renderer').themes.github;
const darkCodeTheme = require('prism-react-renderer').themes.dracula;

const math = require('remark-math');
const katex = require('rehype-katex');

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Hydro - Build for Every Scale',
  tagline: 'Dinosaurs are cool',
  favicon: 'img/favicon.ico',

  // Set the production url of your site here
  url: 'https://hydro.run',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'hydro-project', // Usually your GitHub org/user name.
  projectName: 'hydroflow', // Usually your repo name.

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'throw',

  customFields: {
    'LOAD_PLAYGROUND': process.env.LOAD_PLAYGROUND || false,
  },

  markdown: {
    mermaid: true
  },

  themes: [
    '@docusaurus/theme-mermaid'
  ],

  // Even if you don't use internalization, you can use this field to set useful
  // metadata like html lang. For example, if your site is Chinese, you may want
  // to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  stylesheets: [
    {
      href: 'https://cdn.jsdelivr.net/npm/katex@0.13.24/dist/katex.min.css',
      type: 'text/css',
      integrity:
        'sha384-odtC+0UGzzFL/6PNoE8rX/SPcQDXBJ+uRepguP4QkPCm2LBxH3FA3y+fKSiJ+AmM',
      crossorigin: 'anonymous',
    },
  ],

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl:
            'https://github.com/hydro-project/hydroflow/tree/main/docs/',
          remarkPlugins: [math],
          rehypePlugins: [katex],
        },
        // blog: {
        //   showReadingTime: true,
        //   // Please change this to your repo.
        //   // Remove this to remove the "edit this page" links.
        //   editUrl:
        //     'https://github.com/hydro-project/hydroflow/tree/main/docs/',
        // },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      }),
    ],
  ],

  plugins: [
    [
      '@docusaurus/plugin-ideal-image',
      {
        quality: 75,
        max: 1080,
        min: 480,
        steps: 6,
        disableInDev: false,
      },
    ],
    require.resolve("./wasm-plugin.js")
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      // Replace with your project's social card
      image: 'img/social-card.png',
      colorMode: {
        respectPrefersColorScheme: true,
      },
      navbar: {
        title: 'Hydro',
        logo: {
          alt: 'Hydro',
          src: 'img/hydro-logo.svg',
        },
        items: [
          {
            type: 'dropdown',
            label: 'Docs',
            items: [
              {
                type: 'docSidebar',
                sidebarId: 'hydroflowSidebar',
                label: 'Hydroflow',
              },
              {
                type: 'docSidebar',
                sidebarId: 'hydroflowPlusSidebar',
                label: 'Hydroflow+',
              },
              {
                type: 'docSidebar',
                sidebarId: 'deploySidebar',
                label: 'Hydro Deploy',
              },
              {
                href: 'pathname:///rustdoc/hydroflow/',
                label: 'Rustdoc',
              },
              {
                href: "https://axiomatic-dead-766.notion.site",
                label: 'Algebra Wiki',
              }
            ]
          },
          {
            to: '/playground',
            position: 'left',
            label: 'Playground',
          },
          {
            to: '/research',
            position: 'left',
            label: 'Publications',
          },
          {
            to: '/people',
            position: 'left',
            label: 'People',
          },
          // {to: '/blog', label: 'Blog', position: 'left'},
          {
            href: 'https://github.com/hydro-project/hydroflow',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Docs',
            items: [
              {
                label: 'Hydroflow',
                to: '/docs/hydroflow/',
              },
              {
                label: 'Hydroflow+',
                to: '/docs/hydroflow_plus/',
              },
              {
                label: 'Hydro Deploy',
                to: '/docs/deploy/',
              }
            ],
          },
          {
            title: 'Research Group',
            items: [
              {
                label: 'Publications',
                to: '/research',
              },
              {
                label: 'People',
                to: '/people',
              }
            ],
          },
          {
            title: 'More',
            items: [
              // {
              //   label: 'Blog',
              //   to: '/blog',
              // },
              {
                label: 'GitHub',
                href: 'https://github.com/hydro-project/hydroflow',
              },
            ],
          },
        ],
        copyright: `Hydro is a project in the <a href="https://sky.cs.berkeley.edu">Sky Computing Lab</a> at UC Berkeley. We are grateful to be supported by <a href="https://shv.com">Sutter Hill Ventures</a>.`,
      },
      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
        additionalLanguages: ['rust', 'bash'],
        magicComments: [
          {
            className: 'theme-code-block-highlighted-line',
            line: 'highlight-next-line',
            block: {start: 'highlight-start', end: 'highlight-end'},
          },
          {
            className: 'shell-command-line',
            line: 'shell-command-next-line',
          }
        ]
      },
      algolia: {
        appId: 'C2TSTQAKIC',
        apiKey: '38cef87035f42759bc1dd871e91e06ba',
        indexName: 'hydro'
      },
    }),
    scripts: [
      {
        id: "runllm-widget-script",
        type: "module",
        src: "https://widget.runllm.com",
        "runllm-server-address": "https://api.runllm.com",
        "runllm-assistant-id": "136",
        "runllm-position": "BOTTOM_RIGHT",
        "runllm-keyboard-shortcut": "Mod+j",
        "runllm-preset": "docusaurus",
        "runllm-slack-community-url": "",
        "runllm-name": "Hydro",
        "runllm-theme-color": "#005EEC",
        async: true,
      },
    ],
};

module.exports = config;
