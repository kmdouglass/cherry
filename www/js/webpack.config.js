const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: "./src/index.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bundle.js",
  },
  module: {
    rules: [
      {
        test: /\.js|\.jsx$/,
        exclude: "/node_modules/",
        use: {
          loader: "babel-loader",
          options:{
              presets:[
                "@babel/preset-env",
                ["@babel/preset-react", {"runtime": "automatic"}],
              ]
          }
        }
      },
      {
        test: /\.css$/,
        use: [
          'style-loader',
          'css-loader'
        ]
      }
    ]
  },
  plugins: [
    new CopyWebpackPlugin({
      patterns: [
        { from: "src/index.html", to: "index.html" },
        { from: "src/data/catalog-nk.json", to: "data/catalog-nk.json" }
      ],
    })
  ],
  "experiments": {
    "asyncWebAssembly": true
  }
};
