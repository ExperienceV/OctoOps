#!/bin/bash

npm init -y
npm install -D typescript tsx @types/node
npm install fastify
npx tsc --init
npm pkg set scripts.dev="tsx watch src/index.ts"
npm run dev