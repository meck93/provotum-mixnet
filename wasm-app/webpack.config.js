const HtmlWebpackPlugin = require("html-webpack-plugin");
const path = require("path");
const webpack = require("webpack");

module.exports = {
  entry: "./src/bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  mode: "development",
  plugins: [new HtmlWebpackPlugin()],
  devServer: {
    host: "0.0.0.0",
    port: 8081,
  },
  experiments: {
    asyncWebAssembly: false,
    topLevelAwait: true,
    syncWebAssembly: true,
  },
};
