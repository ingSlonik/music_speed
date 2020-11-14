import vegaEmbed from "vega-embed";

import { version } from "../package.json";

// rust web-view handler
declare let external: {
    invoke: (type: string) => void,
};

console.log(version);

var size = 0;
var data = [];

var progressBar = document.getElementById("progress-bar");
var progressText = document.getElementById("progress-text");

document.getElementById("select").onclick = () => {
    external.invoke("open_dialog");
};

var logElement = document.getElementById("log");

function showChart() {
    vegaEmbed("#vis", {
        $schema: "https://vega.github.io/schema/vega-lite/v4.json",
        width: 500,
        data: { values: data },
        mark: { type: "line", point: true, tooltip: true },
        encoding: {
            x: { field: "time", type: "quantitative" },
            y: { field: "bpm", type: "quantitative" }
        }
    }, { actions: false });
}

// ------------- API -----------------

const api = window as any;

api.start = (s) => {
    size = s;
    data = [];

    progressBar.style.width = "0%";
    progressText.innerText = "0 %";
};

api.step = (time, bpm) => {
    data.push({ time, bpm });

    var progress = data.length / size * 100;

    progressBar.style.width = progress + "%";
    progressText.innerText = progress.toFixed(2) + " %";

    showChart();
};

api.end = () => {
    progressBar.style.width = "100%";
    progressText.innerText = "Done";
};

api.log = (text) => {
    logElement.innerHTML += text + "<br />";
};
