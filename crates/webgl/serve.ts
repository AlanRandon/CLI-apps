import express from "express"
import fs from "node:fs/promises"
import path from "node:path"

// Windows
// const dirname = path.dirname(import.meta.url).replace(/^file:\/\/\//gms, "")
// Codespaces
const dirname = path.dirname(import.meta.url).replace(/^file:\/\//gms, "")

const resolve = (...paths: string[]) => [dirname, ...paths].join("/")

const read = async (...paths: string[]) => {
  return await fs.readFile(resolve(...paths), "utf-8")
}

const indexPage = (await read("index.html")).replace(
  "%SCRIPT%",
  `<script type="module">${await read(".dist", "init.js")}</script>`
)

const app = express()

app.use(express.static(".dist"))

app.get("/", (_request, response) => {
  response.send(indexPage)
})

app.listen(8080, "localhost")

console.log("Started: http://localhost:8080")
