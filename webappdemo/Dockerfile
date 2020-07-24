FROM node:lts

RUN yarn global add serve

WORKDIR /app
COPY *.json /app/
COPY yarn.lock /app/
RUN yarn

COPY src/ /app/src/
COPY public/ /app/public/
RUN yarn build
CMD ["serve", "-l", "80", "-s", "/app/build"]
