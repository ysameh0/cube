const path = require('path');
const fs = require('fs/promises');
const native = require('../dist/js/index');

(async () => {
    native.setupLogger(
        ({ event }) => console.log(event),
        'trace',
    );

    const content = await fs.readFile(path.join(process.cwd(), 'test', 'config.py'), 'utf8');
    console.log('content', {
        content
    });

    const config = await native.pythonLoadConfig(
        content,
      {
          file: 'config.py'
      }
    );

    console.log(config);
})();
