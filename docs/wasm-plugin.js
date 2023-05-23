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
        }
      };
    },
  };
};
