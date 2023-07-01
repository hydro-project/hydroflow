module.exports = function (context, options) {
  return {
    name: 'wasm-docusuarus-plugin',
    // eslint-disable-next-line
    configureWebpack(config, isServer, utils) {
      return {
        experiments: {
          asyncWebAssembly: !isServer,
        },
        module: {
          rules: isServer ? [
            {
              test: /\.wasm$/,
              type: "asset/inline",
            },
          ] : []
        },
        ...(process.env.LOAD_PLAYGROUND !== "1" ? {
          resolve: {
            alias: {
              "website_playground/website_playground_bg.wasm": false,
              "website_playground/website_playground_bg.js": false
            }
          }
        } : {})
      };
    },
  };
};
