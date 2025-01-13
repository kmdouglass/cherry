const CompressionPlugin = require("compression-webpack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require("path");

module.exports = (env, argv) => {
  console.log("argv: ", argv);
  return {
    entry: "./src/index.js",
    output: {
      path: path.resolve(__dirname, "dist"),
      publicPath: argv.mode === "production" ? "/cherry/" : "/",
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
            "style-loader",
            "css-loader"
          ]
        }
      ]
    },
    plugins: [
      new CompressionPlugin(),
      new CopyWebpackPlugin({
        patterns: [
          { from: "src/index.html", to: "index.html" },
          { from: "src/data/initial-materials-data.json", to: "data/initial-materials-data.json" }
          //{ from: "src/data/full-materials-data.json", to: "data/full-materials-data.json" }
        ],
      }),
    ],
    "experiments": {
      "asyncWebAssembly": true
    },
    devServer: {
      static: {
        directory: path.join(__dirname, 'dist')
      },
      devMiddleware: {
        publicPath: '/'
      },
      compress: true // Enable gzip compression for everything served
    }
  }
};
