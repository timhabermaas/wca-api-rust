#! /bin/bash
echo "Importing WCA export"
filename="$(curl --silent https://www.worldcubeassociation.org/results/misc/export.html | grep -E 'WCA_export[0-9_]+\.tsv.zip' -o | head -n1 | tr -d '\n')"
mkdir -p data
echo "Downloading export files"
curl http://www.worldcubeassociation.org/results/misc/${filename} > data/download.zip
unzip -o data/download.zip -d data
