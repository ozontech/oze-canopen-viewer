# GUI Description

![](marks.png)

1. Enter the name of the interface from which the data will be read, e.g., `can0`.
2. You can enter the desired bitrate. If not specified, the current bitrate will be used and reading will proceed as before. If set, and if the current bitrate differs from the desired one, the CAN interface in Linux will be taken down (set link down), the bitrate will be changed, and then the interface will be brought back up (set link up).
3. After modifying fields 1 or 2, click this button to apply the changes.
4. Buttons to select the data packet print format. HEX - hexadecimal representation, bin - binary representation, ASCII - if possible, convert bytes to displayable ASCII characters; if unsuccessful, a `.` will be displayed.
5. `rx true` if the receiving socket is connected.
6. `tx true` if the transmitting socket is connected.
7. Displays statistics.
8. Displays the interface's FPS for debugging purposes.
9. Network load graph showing bits per second over time.
10. Start/stop packet reading.
11. Select/deselect all filter checkboxes.
12. Filters by packet type. The type is determined by the COB-ID. If the checkbox is selected, the packets are read; otherwise, they are ignored.
13. Filter by the hexadecimal representation of COB-ID. Full regex is supported.
14. Filter by nodeID. A number is supported. If a number is set but there is no nodeID in the packet data, the packet is ignored.
15. Filter by the selected data representation. Full regex is supported.
16. Pin the current data filter.
17. Filter settings.
18. Pinned filters.
19. Filtered messages, up to 4096, can be scrolled using the mouse wheel or slider.
20. Delete the pinned filter.

# CLI Arguments

```
OZON CanOpenViewer

Usage: oze-canopen-viewer [OPTIONS]

Options:
  -c, --can <CAN>          
  -b, --bitrate <BITRATE>  
  -h, --help               Print help
  -V, --version            Print version
```

If `--can` is specified, the CAN interface from which the data will be read will be set at startup; otherwise, you need to enter it in the GUI.

If `--bitrate` is specified, the desired bitrate of the CAN interface will be set at startup; otherwise, you need to enter it in the GUI if necessary.