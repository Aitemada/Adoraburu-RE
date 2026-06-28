function getRandomInt(max) {
    return Math.floor(Math.random() * max);
}

let randSong = (a) =>{

let rand = getRandomInt(3)

//  alert(`${rand} hey`)

// const songlist = {0:'usseewa',1:"readymade", 2:"show" };

// alert(`${songlist[rand]} `)

if(rand == 0){
    location.href = "../../pages/404.html" //pages/usseewa/usseewa.html
}else if(rand == 1){
    location.href = "../../pages/404.html" //pages/readymade/readymade.html
}else if(rand == 2){
     location.href = "../../index.html" 
}
return 777
}

// let hideButVal = 1 //is shown    <-- Wnted to add my opinion/commentary for every song, yet there are Deadlines. Maybe someday
// let hideComm = () =>{
// if(hideButVal == 0){
// hideButVal = 1
// document.getElementById("HideCom").innerHTML = "Hide Comment" 
// document.getElementById("comspan").style.display = "flex"
// }else if(hideButVal == 1){
// hideButVal = 0
// document.getElementById("HideCom").innerHTML = "Show Comment" 
// document.getElementById("comspan").style.display = "none" 
// }
// }

let showLyr = () =>{
    document.getElementById("mainCont").style.justifyContent = "flex-start"
    let switcher = document.getElementById('switlyr')
    let killer = document.getElementById('killlyr')
    let lyrsS = document.getElementById('lyrics')
    let vide = document.getElementsByTagName('video')[0]
    vide.style.right = '0'
    $('html,body').scrollTop(0);
    document.body.style.overflowY = "hidden"
    switcher.style.display = "none"
    killer.style.display = 'flex'
    lyrsS.style.display = 'flex'
    // if(screen.width >= 1080){
    lyrsS.style.flexFlow = "wrap";
    lyrsS.style.position = "absolute";
    lyrsS.style.height = "100%";
    lyrsS.style.top = "0";
    //     lyrsS.style.alignSelf = "flex end";
    lyrsS.style.width = "-webkit-fill-available"
    vide.style.position = 'absolute'
    //     killer.style.position = "absolute"
    // }else{vide.style.display = 'none'}
}

let kiLyr = () =>{
    let switcher = document.getElementById('switlyr')
    let killer = document.getElementById('killlyr')
    let lyrsS = document.getElementById('lyrics')
    let vide = document.getElementsByTagName('video')[0]
    // lyrsS.style.flexFlow = "column wrap";
    vide.style.display = 'flex'
    vide.style.position = 'static'
    document.body.style.overflowY = "unset"
    // killer.style.position = "static"
    switcher.style.display = "flex"
    killer.style.display = 'none'
    lyrsS.style.display = 'none'
}

document.getElementById('switlyr').addEventListener("click", showLyr);
document.getElementById('killlyr').addEventListener('click', kiLyr);


let showSponsors = () => location.href = "../../pages/404.html"
let reHome = () => location.href = "../../index.html";
let AdoYT = () => window.open("https://www.youtube.com/channel/UCln9P4Qm3-EAY4aiEPmRwEA", "_blank", "noreferrer", "noopener");
let sngList = () => location.href = "../../pages/Songlist/songlist.html"
let donate = () => location.href = "../../pages/404.html"

//https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/random