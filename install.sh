#!/bin/bash
# Install or update TuringArena on a server

msg() {
    echo "==> $@"
}

gen_secret() {
    head -c42 /dev/urandom | base64
}

PORT="3000"
HOST="localhost"

REPO="${PWD}"
BIN="${HOME}/.local/bin/turingarena"
DEST="${HOME}/.local/share/turingarena"
CONFIG="${HOME}/.config/turingarena/turingarena.conf.json"
CACHE="${HOME}/.cache/turingarena"

msg "Create directories"
mkdir -p "${DEST}/backend"
mkdir -p "${DEST}/frontend"
mkdir -p "${CACHE}"
mkdir -p "${HOME}/.local/bin"
mkdir -p "${HOME}/.config/turingarena"

msg "Build backend"
cd "${REPO}/server/"
npm ci

msg "Install backend"
cp -r * "${DEST}/backend"

msg "Build client"
cd "${REPO}/web/"
npm ci

msg "Install client"
cp -r build "${DEST}/frontend"

msg "Install binary ${BIN}"
tee "${BIN}" << EOF
#!/bin/sh

NODE_PATH="${DEST}/backend/node_modules/" exec node "${DEST}/backend/dist/src/bin/turingarena.js" --config "${CONFIG}" "\$@"
EOF
chmod +x "${BIN}"

msg "Create config file in ${CONFIG}"
tee "${CONFIG}" <<EOF
{
    "db": {
        "storage": "${DEST}/db.sqlite3",
        "dialect": "sqlite"
    },
    "host": "${HOST}",
    "port": ${PORT},
    "secret": "$(gen_secret)",
    "skipAuth": false,
    "cachePath": "${CACHE}",
    "webRoot": "${DIST}/frontend"
}
EOF