
class StyleSheets:
    MODERN_BUTTON = """
        QPushButton {
            background-color: #4CAF50;
            border: none;
            color: white;
            padding: 10px 20px;
            text-align: center;
            text-decoration: none;
            border-radius: 5px;
            font-size: 12pt;
            font-weight: 500;
        }
        QPushButton:hover {
            background-color: #45a049;
        }
        QPushButton:pressed {
            background-color: #3e8e41;
        }
    """

    DISABLED_BUTTON = """
        QPushButton:disabled {
            background-color: #cccccc;
            color: #666666;
        }
    """

    # You can add more stylesheets here
    HEADER_LABEL = """
        QLabel {
            font-size: 18pt;
            font-weight: bold;
            color: #333333;
        }
    """
