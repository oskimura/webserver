FROM node:19-buster-slim

WORKDIR /usr/src/app/
COPY . .
WORKDIR /usr/src/app/rust-front
COPY ./rust-front/package*.json ./

RUN npm install axios --save
RUN npm install 

EXPOSE 3000
CMD [ "npm", "run", "dev" ]
