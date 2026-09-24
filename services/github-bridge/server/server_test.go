package server

import (
	"net/http"
	"net/http/httptest"
	"os"
	"testing"

	"github.com/gin-gonic/gin"
)

func TestCORSMiddleware_DefaultWildcard(t *testing.T) {
	gin.SetMode(gin.TestMode)
	os.Unsetenv("CORS_ALLOWED_ORIGIN")

	r := gin.New()
	r.Use(CORSMiddleware())
	r.GET("/test", func(c *gin.Context) {
		c.String(http.StatusOK, "ok")
	})

	req, _ := http.NewRequest("GET", "/test", nil)
	w := httptest.NewRecorder()
	r.ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("Expected status 200, got %d", w.Code)
	}

	origin := w.Header().Get("Access-Control-Allow-Origin")
	if origin != "*" {
		t.Errorf("Expected Access-Control-Allow-Origin to be '*', got '%s'", origin)
	}

	credentials := w.Header().Get("Access-Control-Allow-Credentials")
	if credentials != "" {
		t.Errorf("Expected Access-Control-Allow-Credentials to be empty when origin is '*', got '%s'", credentials)
	}
}

func TestCORSMiddleware_SpecificOrigin(t *testing.T) {
	gin.SetMode(gin.TestMode)
	os.Setenv("CORS_ALLOWED_ORIGIN", "https://app.example.com")
	defer os.Unsetenv("CORS_ALLOWED_ORIGIN")

	r := gin.New()
	r.Use(CORSMiddleware())
	r.GET("/test", func(c *gin.Context) {
		c.String(http.StatusOK, "ok")
	})

	req, _ := http.NewRequest("GET", "/test", nil)
	w := httptest.NewRecorder()
	r.ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("Expected status 200, got %d", w.Code)
	}

	origin := w.Header().Get("Access-Control-Allow-Origin")
	if origin != "https://app.example.com" {
		t.Errorf("Expected Access-Control-Allow-Origin to be 'https://app.example.com', got '%s'", origin)
	}

	credentials := w.Header().Get("Access-Control-Allow-Credentials")
	if credentials != "true" {
		t.Errorf("Expected Access-Control-Allow-Credentials to be 'true' for specific origin, got '%s'", credentials)
	}
}

func TestCORSMiddleware_PreflightOptions(t *testing.T) {
	gin.SetMode(gin.TestMode)
	os.Unsetenv("CORS_ALLOWED_ORIGIN")

	r := gin.New()
	r.Use(CORSMiddleware())
	r.GET("/test", func(c *gin.Context) {
		c.String(http.StatusOK, "ok")
	})

	req, _ := http.NewRequest("OPTIONS", "/test", nil)
	w := httptest.NewRecorder()
	r.ServeHTTP(w, req)

	if w.Code != http.StatusNoContent {
		t.Errorf("Expected status 204 for OPTIONS preflight, got %d", w.Code)
	}

	origin := w.Header().Get("Access-Control-Allow-Origin")
	if origin != "*" {
		t.Errorf("Expected Access-Control-Allow-Origin to be '*', got '%s'", origin)
	}
}
