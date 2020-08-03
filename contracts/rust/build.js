#!/usr/bin/env node
const sh = require('shelljs')
const path = require('path')

sh.fatal = true // same as "set -e"

sh.cd(__dirname)

// Note: see flags in ./cargo/config
sh.exec('cargo build --target wasm32-unknown-unknown --release')

const outdir = '../../out'
sh.mkdir('-p', outdir)

sh.ls('./target/wasm32-unknown-unknown/release/*.wasm').map(src => {
  const output = path.basename(src)
    .replace('.wasm', '-rs.wasm')
    .replace(/_/g, '-')

  console.log(`\ncopying [ ${src} ] to [ out/${output} ]`);

  sh.cp(src, `${outdir}/${output}`)
})
