const path = require("path");

module.exports = {
    entry: path.resolve(__dirname, "src", "index.ts"),
    resolve: {
        extensions: [ ".tsx", ".ts", ".js" ],
        //mainFields: [ "browser", "main", "module" ],
    },
    target: "es5",
    devtool: "inline-source-map",
    output: {
        filename: "bundle.js",
        path: path.resolve(__dirname, "dist"),
    },
    devServer: {
        contentBase: path.join(__dirname, "dist"),
        compress: true,
        port: 9000
    },
    module: {
        rules: [
            /* BABEL
            {
                test: /\.(js|ts)$/,
                exclude: /(node_modules|bower_components)/,
                use: {
                    loader: "babel-loader",
                    options: {
                        presets: ["@babel/preset-env"]
                    }
                }
            },
            {
                test: /(d3-|delaunator|semver|vega-).*\.js$/,
                use: {
                    loader: "babel-loader",
                    options: {
                        presets: ["@babel/preset-env"]
                    }
                }
            } */
            // SWC
            {
                test: /\.ts$/,
                exclude: /node_modules/,
                use: {
                    loader: "swc-loader",
                    options: {
                        jsc: {
                            target: "es5",
                            parser: {
                                syntax: "typescript"
                            }
                        }
                    }
                }
            },
        ]
    }
};