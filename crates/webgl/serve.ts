import express from "express"
import fs from "node:fs/promises"
import path from "node:path"

const dirname = path.dirname(import.meta.url).replace(/^file:\/\/\//gms, "")

const read = async (...paths: string[]) => {
  return await fs.readFile(path.resolve(dirname, ...paths), "utf-8")
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

app.listen(3000, "localhost")

console.log("Started: http://localhost:3000")
