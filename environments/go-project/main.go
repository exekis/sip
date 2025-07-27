package main

import (
    "fmt"
    "github.com/gin-gonic/gin"
    "github.com/gorilla/mux"
)

func main() {
    fmt.Println("hello from go test project")
    fmt.Println("gin and mux imported successfully")
    
    // placeholder usage to avoid import warnings
    _ = gin.Default
    _ = mux.NewRouter
}
