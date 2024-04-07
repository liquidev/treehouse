import { addElement } from "treehouse/sandbox.js";

export function showTable(data, schema) {
    let table = document.createElement("table");

    let thead = table.appendChild(document.createElement("thead"));
    for (let column of schema.columns) {
        let th = thead.appendChild(document.createElement("th"));
        th.textContent = column;
    }

    let tbody = table.appendChild(document.createElement("tbody"));
    for (let row of data) {
        let tr = tbody.appendChild(document.createElement("tr"));
        for (let column of schema.columns) {
            let td = tr.appendChild(document.createElement("td"));
            td.textContent = `${row[column]}`;
        }
    }

    addElement(table);
}
