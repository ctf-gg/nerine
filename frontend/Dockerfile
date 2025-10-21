FROM node:24-alpine AS base
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
COPY . /app
WORKDIR /app

FROM base AS build
ENV CI=true
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm run build
RUN pnpm prune --prod

FROM base
COPY --from=build /app/build /app
ENV PORT=3334
ENV NODE_ENV=production
EXPOSE 3334
CMD [ "node", "./index.js" ]
