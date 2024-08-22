from PyQt6.QtGui import QPixmap
from PyQt6.QtCore import Qt, QSize
import os

class PixmapCache:
    def __init__(self):
        self.cache = {}

    def getPixmap(self, imagePath, targetWidth, targetHeight):
        """
        Get a QPixmap from the cache or create a new one if it doesn't exist.
        
        :param image_path: Path to the image file
        :param target_size: Tuple or QSize specifying the target size (width, height)
        :return: QPixmap object
        """
        targetSize = QSize(targetWidth, targetHeight)
          
        cacheKey = (imagePath, targetSize.width(), targetSize.height())
        
        if cacheKey in self.cache:
            return self.cache[cacheKey]
        
        if not os.path.exists(imagePath):
            print(f"Warning: Image file not found: {imagePath}")
            return QPixmap()
        
        pixmap = QPixmap(imagePath)
        if pixmap.isNull():
            print(f"Warning: Failed to load image: {imagePath}")
            return QPixmap()
        
        scaledPixmap = pixmap.scaled(
            targetSize,
            aspectRatioMode=Qt.AspectRatioMode.KeepAspectRatio,
            transformMode=Qt.TransformationMode.SmoothTransformation)
        
        self.cache[cacheKey] = scaledPixmap
        return scaledPixmap

    def clearCache(self):
        self.cache.clear()
