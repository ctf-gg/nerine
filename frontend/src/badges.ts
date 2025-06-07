interface BadgeInfo {
  icon: string;
  name: string;
  description: string;
}

export const badges: { [k: string]: BadgeInfo } = {
  pwn: {
    icon: "pwn.png",
    name: "Poison Puffcap",
    description:
      '"Hello, Mr. High Major Commodore of the First Legion Third Multiplication Double Admiral Artillery Vanguard Company Sir!"',
  },
  rev: {
    icon: "rev.png",
    name: "Severed Octopus Tentacle",
    description: "I don't think the owner is going to be very happy with this",
  },
  web: {
    icon: "web.png",
    name: "Badge Not Found",
    description:
      "I couldn't find the image in chromium source so I resorted to copying it pixel by pixel",
  },
  crypto: {
    icon: "crypto.png",
    name: "Cat Ears",
    description: "Meow.",
  },
  misc: {
    icon: "misc.png",
    name: "Pickle",
    description: "Dill with it.",
  },
};
