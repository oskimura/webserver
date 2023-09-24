FROM node:19-buster-slim

WORKDIR /usr/src/app/rust-front
COPY ./rust-front/package*.json ./

RUN npm install

COPY ./rust-front/ .


EXPOSE 3000
CMD [ "npm", "run", "dev" ]
