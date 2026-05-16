package main

import (
	"log"
	"net/http"
)

func main() {
	registry := NewSpatialRegistry()
	fields := NewPotentialFieldRegistry()
	hub := NewHub()

	registry.AddEntity(0, 0, 100, 1.0)
	fields.AddEmitter(EmitterGaussian, 0, 0, -20, 50000.0, 50.0)

	go hub.Run()
	go StartPhysicsLoop(registry, fields, hub)

	http.HandleFunc("/ws", func(w http.ResponseWriter, r *http.Request) {
		conn, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			log.Println("upgrade error:", err)
			return
		}
		client := &Client{hub: hub, conn: conn, send: make(chan []byte, 256)}
		hub.register <- client
		go client.WritePump()
		go client.ReadPump()
	})

	log.Println("Go Spatial Core listening on :8080")
	if err := http.ListenAndServe(":8080", nil); err != nil {
		log.Fatal("ListenAndServe:", err)
	}
}
