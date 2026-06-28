function getRandomInt(max) {
    return Math.floor(Math.random() * max);
}

let randSong = (a) =>{

let rand = getRandomInt(3)

//  alert(`${rand} hey`)

// const songlist = {0:'usseewa',1:"readymade", 2:"show" };

// alert(`${songlist[rand]} `)

if(rand == 0){
    location.href = "404.html" //pages/usseewa/usseewa.html
}else if(rand == 1){
    location.href = "404.html" //pages/readymade/readymade.html
}else if(rand == 2){
    location.href = "../pages/show/show.html"
}
return 777
}

let showSponsors = () => location.href = "404.html"
let reHome = () => location.href = "../index.html";
let AdoYT = () => window.open("https://www.youtube.com/channel/UCln9P4Qm3-EAY4aiEPmRwEA", "_blank", "noreferrer", "noopener");
let sngList = () => location.href = "Songlist/songlist.html"
let donate = () => location.href = "404.html"

//https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/random