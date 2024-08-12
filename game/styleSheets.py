
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

    RESOURCE_LIST_TEXT = """
        QWidget {
            font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
            font-size: 14pt;
            color: #2c3e50;
        }
        QLabel {
            font-weight: 400;
            line-height: 1.4;
        }
    """
    
    BUILDING_TITLE = """
        QWidget {
            font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
            font-size: 14pt;
            color: #2c3e50;
        }
        QLabel {
            font-weight: bold;
            line-height: 1.4;
        }
    """
    
    BUILDING_RESOURCE_LIST = """
        QWidget {
            font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
            font-size: 12pt;
            color: #2c3e50;
        }
        QLabel {
            font-weight: bold;
            line-height: 1.4;
        }
    """
    
    BUILDING_DESCRIPTION = """
        QWidget {
            font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
            font-size: 12pt;
            color: #2c3e50;
        }
        QLabel {
            font-weight: 400;
            line-height: 1.4;
        }
    """
