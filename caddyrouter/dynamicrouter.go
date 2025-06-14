// vim: se ts=4 sw=4 et ai:
package caddyrouter

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"sync"

	"github.com/caddyserver/caddy/v2"
	"github.com/caddyserver/caddy/v2/modules/caddyhttp"
)

func init() {
    caddy.RegisterModule(DynamicRouter{})
    caddy.RegisterModule(DynamicRouterHandler{})
    caddy.RegisterModule(DynamicRouterAdmin{})
}

type DynamicRouter struct {
    routes   map[string]string
    routesMu sync.RWMutex
}

var (
    _ caddy.Module = (*DynamicRouter)(nil)
    _ caddy.App = (*DynamicRouter)(nil)
    _ caddy.Provisioner = (*DynamicRouter)(nil)
)

// Module

func (DynamicRouter) CaddyModule() caddy.ModuleInfo {
    return caddy.ModuleInfo{
        ID: "dynamic_router",
        New: func() caddy.Module { return new(DynamicRouter) },
    }
}

// Provisioner

func (rt *DynamicRouter) Provision(ctx caddy.Context) error {
    rt.routes   = make(map[string]string)
    rt.routesMu = sync.RWMutex{}
    return nil
}

// App

func (rt *DynamicRouter) Start() error {
    return nil
}

func (rt *DynamicRouter) Stop() error {
    return nil
}

// etc

func (rt *DynamicRouter) lookup(route string) (string, bool) {
    rt.routesMu.RLock()
    res, ok := rt.routes[route]
    rt.routesMu.RUnlock()
    return res, ok
}

func (rt *DynamicRouter) add(route string, upstream string) {
    rt.routesMu.Lock()
    rt.routes[route] = upstream
    rt.routesMu.Unlock()
}

func (rt *DynamicRouter) del(route string) {
    rt.routesMu.Lock()
    delete(rt.routes, route)
    rt.routesMu.Unlock()
}

type DynamicRouterAdmin struct {
    ctx caddy.Context
    rt *DynamicRouter
}

var (
    _ caddy.Module = (*DynamicRouterAdmin)(nil)
    _ caddy.Provisioner = (*DynamicRouterAdmin)(nil)
    _ caddy.AdminRouter = (*DynamicRouterAdmin)(nil)
)

func (DynamicRouterAdmin) CaddyModule() caddy.ModuleInfo {
    return caddy.ModuleInfo{
        ID: "admin.api.dynamic_router",
        New: func() caddy.Module { return new(DynamicRouterAdmin) },
    }
}

func (a *DynamicRouterAdmin) Provision(ctx caddy.Context) error {
    a.ctx = ctx
    app, err := ctx.App("dynamic_router")
    if err != nil {
        return err
    }

    a.rt = app.(*DynamicRouter)
    return nil
}

func (a *DynamicRouterAdmin) Routes() []caddy.AdminRoute {
    return []caddy.AdminRoute{
        {
            Pattern: "/dynamic-router/",
            Handler: caddy.AdminHandlerFunc(a.handleAPIEndpoints),
        },
    }
}

func (a *DynamicRouterAdmin) handleAPIEndpoints(w http.ResponseWriter, r *http.Request) error {
    uri := strings.TrimPrefix(r.URL.Path, "/dynamic-router/")
    parts := strings.Split(uri, "/")
    switch {
    case len(parts) == 1 && strings.EqualFold(parts[0], "add"):
        return a.handleAdd(w, r)
    case len(parts) == 1 && strings.EqualFold(parts[0], "delete"):
        return a.handleDelete(w, r)
    }
    return caddy.APIError{
        HTTPStatus: http.StatusNotFound,
        Err:        fmt.Errorf("resource not found: %v", r.URL.Path),
    }
}

type addRequest struct {
    Host     string `json:"host"`
    Upstream string `json:"upstream"`
}

func (a *DynamicRouterAdmin) handleAdd(w http.ResponseWriter, r *http.Request) error {
    if r.Method != http.MethodPost {
        return caddy.APIError{
            HTTPStatus: http.StatusMethodNotAllowed,
        }
    }

    var ar addRequest
    decoder := json.NewDecoder(r.Body)
    defer r.Body.Close()
    if err := decoder.Decode(&ar); err != nil {
        return caddy.APIError{
            HTTPStatus: http.StatusBadRequest,
            Err:        fmt.Errorf("payload unintelligible: %v", err),
        }
    }

    a.rt.add(ar.Host, ar.Upstream)
    w.WriteHeader(http.StatusOK)
    return nil
}

type delRequest struct {
    Host string `json:"host"`
}

func (a *DynamicRouterAdmin) handleDelete(w http.ResponseWriter, r *http.Request) error {
    if r.Method != http.MethodPost {
        return caddy.APIError{
            HTTPStatus: http.StatusMethodNotAllowed,
        }
    }

    var dr delRequest
    decoder := json.NewDecoder(r.Body)
    defer r.Body.Close()
    if err := decoder.Decode(&dr); err != nil {
        return caddy.APIError{
            HTTPStatus: http.StatusBadRequest,
            Err:        fmt.Errorf("payload unintelligible: %v", err),
        }
    }

    a.rt.del(dr.Host)
    w.WriteHeader(http.StatusOK)
    return nil
}

type DynamicRouterHandler struct {
    ctx caddy.Context
    rt *DynamicRouter
}

var (
    _ caddy.Module = (*DynamicRouterHandler)(nil)
    _ caddy.Provisioner = (*DynamicRouterHandler)(nil)
)

func (DynamicRouterHandler) CaddyModule() caddy.ModuleInfo {
    return caddy.ModuleInfo{
        ID: "http.handlers.dynamic_router",
        New: func() caddy.Module { return new(DynamicRouterHandler) },
    }
}

func (rt *DynamicRouterHandler) Provision(ctx caddy.Context) error {
    rt.ctx = ctx

    app, err := ctx.App("dynamic_router")
    if err != nil {
        return err
    }

    rt.rt = app.(*DynamicRouter)
    return nil
}

func (rt *DynamicRouterHandler) ServeHTTP(w http.ResponseWriter, r *http.Request, next caddyhttp.Handler) error {
    hostHeader := r.Host

    upstream, ok := rt.rt.lookup(hostHeader)
    if !ok {
        http.Error(w, "no upstream found", http.StatusNotFound)
        return next.ServeHTTP(w, r)
    }

    caddyhttp.SetVar(r.Context(), "dynamic.upstream", upstream)

    return next.ServeHTTP(w, r)
}
