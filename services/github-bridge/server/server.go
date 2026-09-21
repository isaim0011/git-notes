package server

import (
	"fmt"
	"net/http"
	"strconv"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/git-notes/github-bridge/notes"
)

func CORSMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		c.Writer.Header().Set("Access-Control-Allow-Origin", "*")
		c.Writer.Header().Set("Access-Control-Allow-Credentials", "true")
		c.Writer.Header().Set("Access-Control-Allow-Headers", "Content-Type, Content-Length, Accept-Encoding, X-CSRF-Token, Authorization, accept, origin, Cache-Control, X-Requested-With")
		c.Writer.Header().Set("Access-Control-Allow-Methods", "POST, OPTIONS, GET, PUT, DELETE")

		if c.Request.Method == "OPTIONS" {
			c.AbortWithStatus(204)
			return
		}

		c.Next()
	}
}

func AuthMiddleware(token string) gin.HandlerFunc {
	return func(c *gin.Context) {
		if token == "" {
			c.Next()
			return
		}

		authHeader := c.GetHeader("Authorization")
		if authHeader == "" {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "Authorization header required"})
			return
		}

		parts := strings.Split(authHeader, " ")
		if len(parts) != 2 || parts[0] != "Bearer" || parts[1] != token {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "Invalid token"})
			return
		}

		c.Next()
	}
}

func Run(port string, reader *notes.Reader) error {
	r := gin.Default()
	r.Use(CORSMiddleware())
	
	// Can configure a static token for the bridge API if needed
	// r.Use(AuthMiddleware("some-token"))

	r.GET("/health", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	api := r.Group("/api")
	{
		api.GET("/notes", func(c *gin.Context) {
			file := c.Query("file")
			lineStr := c.Query("line")
			namespace := c.Query("namespace")

			allNotes, err := reader.ReadNotesJSON(namespace)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}

			// Filter notes
			var filtered []notes.Note
			for _, n := range allNotes {
				if file != "" && n.File != file {
					continue
				}
				if lineStr != "" {
					line, _ := strconv.Atoi(lineStr)
					if n.Line != line {
						continue
					}
				}
				filtered = append(filtered, n)
			}

			c.JSON(http.StatusOK, filtered)
		})

		api.POST("/notes", func(c *gin.Context) {
			var note notes.Note
			if err := c.BindJSON(&note); err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
				return
			}

			if err := reader.WriteNote(note); err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}

			c.JSON(http.StatusOK, gin.H{"status": "success"})
		})
	}

	return r.Run(fmt.Sprintf(":%s", port))
}
