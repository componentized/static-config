#!/bin/bash

set -e;

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)

repository="${1}"
version="${2}"
# replace forbidden characters for the tag
tag=$(echo "${version}" | sed 's/[^a-zA-Z0-9_.\-]/--/g')
revision=$(git rev-parse HEAD)
if [[ $(git status --porcelain) ]] ; then
  revision="${revision}+dirty"
fi

publish_oci() {
    local file="${1}"
    local component=$(basename "${file}" .wasm)

    digest=$(
      wkg oci push \
        --annotation "org.opencontainers.image.title=${component}" \
        --annotation "org.opencontainers.image.version=${version}" \
        --annotation "org.opencontainers.image.source=https://github.com/componentized/static-config.git" \
        --annotation "org.opencontainers.image.revision=${revision}" \
        --annotation "org.opencontainers.image.licenses=Apache-2.0" \
        "${repository}/${component}:${tag}" \
        "${file}" \
        2>&1 \
			| tee /dev/stderr \
			| grep -o 'sha256:[a-f0-9]\{64\}' \
    )

    cosign sign --yes "${repository}/${component}:${tag}@${digest}"
}

publish_oci "${SCRIPT_DIR}/lib/factory.wasm"
