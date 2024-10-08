
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
    
    ICON_LIST_TEXT = """
        QWidget {
            font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
            font-size: 13pt;
            color: #2c3e50;
            font-weight: 400;
            line-height: 1.4;
        }
    """

    RESOURCE_LIST_TEXT = """
        QWidget {
            font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
            font-size: 14pt;
            color: #2c3e50;
            font-weight: 400;
            line-height: 1.4;
        }
    """
    
    GAME_TITLE = """
        QWidget {
            font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
            font-size: 16pt;
            color: #2c3e50;
            font-weight: bold;
            line-height: 1.4;
        }
    """
    
    BUILDING_TITLE = """
        QWidget {
            font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
            font-size: 14pt;
            color: #2c3e50;
            font-weight: bold;
            line-height: 1.4;
        }
    """

    GENERAL_12PT = """
        QWidget {
            font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
            font-size: 12pt;
            color: #2c3e50;
            line-height: 1.4;
        }
    """
    
    GENERAL_12PT_BOLD = """
        QWidget {
            font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
            font-size: 12pt;
            color: #2c3e50;
            font-weight: bold;
            line-height: 1.4;
        }
    """
    
    SELECTED_BUTTON = """
        QWidget {
            font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
            font-size: 14pt;
            color: #2c3e50;
            background-color: #45a049;
            font-weight: bold;
            line-height: 1.4;
        }
    """
    
    BUILDING_RESOURCE_LIST = """
        QWidget {
            font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
            font-size: 12pt;
            color: #2c3e50;
            font-weight: bold;
            line-height: 1.4;
        }
    """

    BUILDING_RESOURCE_LIST_RED = """
        QWidget {
            font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
            font-size: 12pt;
            color: #8B0000;
            font-weight: bold;
            line-height: 1.4;
        }
    """
    
    BUILDING_DESCRIPTION = """
        QWidget {
            font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
            font-size: 12pt;
            color: #2c3e50;
            font-weight: 400;
            line-height: 1.4;
        }
    """
    
    DIALOG_BUTTON = """
            QPushButton {
                background-color: #2980b9;
                color: white;
                border: none;
                padding: 10px;
                font-size: 16px;
                font-weight: bold;
                border-radius: 5px;
                min-width: 100px;
            }
            QPushButton:hover {
                background-color: #2c3e50;
            }
        """
        
    PROGRESS_BAR_RED = """
    QWidget {
            font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
            font-size: 12pt;
            color: #2c3e50;
            font-weight: bold;
            line-height: 1.4;
        }
    QProgressBar::chunk {
        background-color: #FFA0A0;
    }
    """
    
    PROGRESS_BAR_GREEN = """
    QWidget {
            font-family: 'Roboto', 'Segoe UI', 'Helvetica', sans-serif;
            font-size: 12pt;
            color: #2c3e50;
            font-weight: bold;
            line-height: 1.4;
        }
    QProgressBar::chunk {
        background-color: #90EE90;
    }
    """



