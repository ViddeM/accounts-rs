const next = require('next')
const {createProxyMiddleware} = require('http-proxy-middleware')
const express = require('express')

const PORT = process.env.PORT || 3000;
const DEV = process.env.NODE_ENV !== 'production';

const app = next({dev: DEV});
const handle = app.getRequestHandler();

app.prepare().then(() => {
    const app = express()

    app.use('/api', createProxyMiddleware({
        target: process.env.BACKEND_ADDRESS,
        pathRewrite: {'^/api': '/api',},
        ws: false,
    }))

    app.use((req, res) => {
        return handle(req, res)
    })

    app.listen(PORT, err => {
        if (err) throw err;
        console.log(`> Ready on http://localhost:${PORT}`);
    })
})