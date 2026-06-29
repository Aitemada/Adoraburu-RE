# Adoraburu_RE - REST API-revived song lyrics website 

An old static website revamped and reborn with a help of REST API built on Rust!

## Overview & Features
The core of the project is a web site on a pure HTML/CSS/JS stack. The API allows you to add, modify and easily manage/modify the pages on the site
The pages feature song's lyrics, video, and three suggestion boxes at the bottom of the page, which are generated automatically when you create new pages and feature actual, working links to existing song pages. The pages are thematically adjustable via CSS manipulations. All of this can be modified through API calls!

## Tech Stack & Environment
Language & Framework: Rust, Axum, Tokio, Serde
Deployment: Docker, Kubernetes (Minikube).
Observability: Prometheus/Grafana/Loki

## File System & Kubernetes Volumes
There are three main folders in the project:
* AdoRaburu_RE - The main Frontend directory. It contains starter the pages.
* adoraburu_re-api - API project files. Use this folder to build the Docker image.
* adora-k8s-files - All the Kubernetes deployments.

**The API shall operate in the strictly stated Adoraburu_RE/ folder you need to copy onto your machine. State the folder's name within the Install-Adorabu.sh script instead of {SITE-FOLDERPTH} placeholder!** 

The site's structure is the following: main directory contains the landing page files, and /pages directory contains all the other pages' files. /Images and /videos directories contain the respective images and videos used on the pages. All the new pages' directories by default are created as 'songname/', and their resources are bound to '../Images/{songname.jpg}' and '../videos/{songname.mp4}' files. Yet, if you need to update the contents, you simply upload it through an API-call, which would upload the files the right way, reducing manual labor to minimum! 

API utilizes a exstg-songs.json file created locally to manage existing songnames, their lyrics and tooltips for when the song is referenced in a suggestion box. This file can be accessed and re-uploaded through API calls.

## Setup & Deployment
Copy all the directories into any location on your system you see fit. Remember this path, and replace all the *"{SITE-FOLDERPTH}"* markers with it. 
*Example: you copied the directories to /home/myusr/Projects/sitefiles/. Your {SITE-FOLDERPTH} = /home/myusr/Projects/sitefiles!*

In order to start the site, you need to boot the minikube cluster first:
```bash
minikube start --driver=docker
```
After the cluster is started and running, run the *Install-Adorabu.sh* script. It will automatically build the image for the API, mount all needed PVs/PVCs and install the system's components. You can use this script to reinstall the system as well if needed.

The frontend will be accessible via adoraburu.utae/ url in your browser. All the *"api/{...}"* endpoints calls are accessible through this link!

## Quick Start
To add a completely new song to the platform via the API, follow this sequence:

Create the page: POST /api/add-song/{songname}

Upload assets: POST /api/upload/video/{songname} & POST /api/upload/image/{songname}/bg

Push lyrics: POST /api/lyrics/{songname}

Update Suggestions: POST /api/update-suggestions/all to interlink the new page across the site.

## Endpoints Reference
`/api/songs`(GET) - Retrieves the entire JSON metadata map of all currently registered songs.

`/api/songs`(PUT) - Overwrites the entire JSON metadata map with a provided JSON payload.

`/api/add-song/:name`(POST) - Duplicates the template folder, renames files, injects the song name into the HTML, and registers the new song.

`/api/random-song`(GET) - Returns a randomly selected song name from the active metadata pool.

`/api/update-songlist`(POST) - Rebuilds the central songlist.html page's unordered list based on the currently registered songs.

`/api/update-suggestions/all`(POST) - Calculates 3 random distinct song suggestions for every existing song page and injects them into their HTML.

`/api/update-suggestions/:songname`(POST) - Calculates 3 random distinct song suggestions and injects them into the HTML of a specific song.

`/api/lyrics/:songname`(POST) - Updates the song's lyrics in the JSON metadata and immediately overwrites the `<pre id="lyrics">` block in its HTML.

`/api/css/:songname`(GET) - Retrieves the raw CSS text from a specific song's stylesheet.

`/api/css/:songname`(POST) - Overwrites the specified song's CSS stylesheet with the provided raw CSS payload.

`/api/upload/video/:songname`(POST) - Uploads an MP4 file to the /videos/ folder. Use the query ?force=true to overwrite an existing video.

`/api/upload/image/:songname/:img_type`(POST) - Uploads an image as either bg (Background) or thumbic (Thumbnail). Use the query ?force=true to overwrite an existing image.

`/metrics`(GET) - Exposes application metrics (like songs_added_total) for Prometheus scraping.

`/api/health`(GET) - Returns a simple "OK" to verify the API is alive and responsive.

## Legal & Copyright Notice

All third-party multimedia assets (images, videos, and song lyrics) served by this API are the property of their respective owners and are included strictly for non-commercial, demonstrational purposes. 

The open-source license applied to this repository covers exclusively the source code, architecture, and infrastructure manifests authored by me.
